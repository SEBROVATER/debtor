use std::future::Future;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use debtor_application::SupervisorReadiness;
use debtor_web::submission_tokens::SubmissionTokenCleanup;
use sqlx::SqlitePool;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use tower_sessions::ExpiredDeletion;

use crate::composition::BuiltApp;

const CLEANUP_INTERVAL: Duration = Duration::from_mins(5);
const HTTP_DRAIN_TIMEOUT: Duration = Duration::from_secs(10);
const CLEANUP_STOP_TIMEOUT: Duration = Duration::from_secs(5);
const WAL_CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(5);
const POOL_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(crate) struct CleanupHealth(Arc<AtomicBool>);

impl CleanupHealth {
    pub(crate) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }

    fn mark_unhealthy(&self) -> bool {
        self.0
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

impl SupervisorReadiness for CleanupHealth {
    fn is_healthy(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct ShutdownEvents {
    checkpoint_started: Arc<AtomicBool>,
}

#[cfg(test)]
impl ShutdownEvents {
    pub(crate) fn checkpoint_started(&self) -> bool {
        self.checkpoint_started.load(Ordering::Acquire)
    }

    fn mark_checkpoint_started(&self) {
        self.checkpoint_started.store(true, Ordering::Release);
    }
}

#[derive(Clone)]
pub(crate) struct DispatchedMutationRegistry {
    accepting: Arc<AtomicBool>,
    active: Arc<AtomicUsize>,
    empty: Arc<Notify>,
    observed_empty: Arc<AtomicBool>,
}

impl Default for DispatchedMutationRegistry {
    fn default() -> Self {
        Self {
            accepting: Arc::new(AtomicBool::new(true)),
            active: Arc::new(AtomicUsize::new(0)),
            empty: Arc::new(Notify::new()),
            observed_empty: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[allow(dead_code)]
pub(crate) struct MutationLease {
    active: Arc<AtomicUsize>,
    empty: Arc<Notify>,
}

impl DispatchedMutationRegistry {
    pub(crate) fn close(&self) {
        self.accepting.store(false, Ordering::Release);
        self.empty.notify_waiters();
    }

    #[allow(dead_code)]
    pub(crate) fn try_register(&self) -> Option<MutationLease> {
        if !self.accepting.load(Ordering::Acquire) {
            return None;
        }

        self.active.fetch_add(1, Ordering::AcqRel);
        if !self.accepting.load(Ordering::Acquire) {
            self.release();
            return None;
        }

        Some(MutationLease {
            active: self.active.clone(),
            empty: self.empty.clone(),
        })
    }

    pub(crate) async fn wait_until_empty(&self) {
        self.observed_empty.store(true, Ordering::Release);
        loop {
            let notified = self.empty.notified();
            if self.active.load(Ordering::Acquire) == 0 {
                return;
            }
            notified.await;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn observed_empty(&self) -> bool {
        self.observed_empty.load(Ordering::Acquire)
    }

    #[allow(dead_code)]
    fn release(&self) {
        if self.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.empty.notify_waiters();
        }
    }
}

impl Drop for MutationLease {
    fn drop(&mut self) {
        if self.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.empty.notify_waiters();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShutdownTrigger {
    Signal,
    SignalFailure,
    ReadinessFailure,
    CleanupFailure,
    HttpFailure,
    CheckpointFailure,
    PoolCloseFailure,
}

impl ShutdownTrigger {
    fn is_fatal(self) -> bool {
        !matches!(self, Self::Signal)
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Signal => "signal",
            Self::SignalFailure => "signal_failure",
            Self::ReadinessFailure => "readiness_failure",
            Self::CleanupFailure => "cleanup_failure",
            Self::HttpFailure => "http_failure",
            Self::CheckpointFailure => "checkpoint_failure",
            Self::PoolCloseFailure => "pool_close_failure",
        }
    }
}

#[derive(Debug, Default)]
struct ShutdownState {
    first: Option<ShutdownTrigger>,
    fatal_triggers: Vec<ShutdownTrigger>,
}

#[derive(Clone, Default)]
pub(crate) struct ShutdownCoordinator {
    state: Arc<Mutex<ShutdownState>>,
    notify: Arc<Notify>,
}

#[derive(Debug)]
pub(crate) struct ShutdownOutcome {
    pub(crate) first: Option<ShutdownTrigger>,
    pub(crate) fatal_triggers: Vec<ShutdownTrigger>,
}

impl ShutdownCoordinator {
    pub(crate) async fn request(&self, trigger: ShutdownTrigger) {
        self.request_inner(trigger, false).await;
    }

    pub(crate) async fn request_if_unrequested(&self, trigger: ShutdownTrigger) -> bool {
        self.request_inner(trigger, true).await
    }

    async fn request_inner(&self, trigger: ShutdownTrigger, only_if_unrequested: bool) -> bool {
        let mut state = self.state.lock().await;
        let first = state.first.is_none();
        if only_if_unrequested && !first {
            return false;
        }
        if first {
            state.first = Some(trigger);
        }
        let fatal_recorded = trigger.is_fatal() && !state.fatal_triggers.contains(&trigger);
        if fatal_recorded {
            state.fatal_triggers.push(trigger);
        }
        drop(state);
        if first {
            tracing::info!(
                target: "debtor.runtime",
                event = "shutdown_triggered",
                trigger = trigger.name(),
            );
            if trigger.is_fatal() {
                tracing::warn!(
                    target: "debtor.runtime",
                    event = "shutdown_failure",
                    trigger = trigger.name(),
                );
            }
        } else if fatal_recorded {
            tracing::warn!(
                target: "debtor.runtime",
                event = "shutdown_failure",
                trigger = trigger.name(),
            );
        }
        self.notify.notify_waiters();
        true
    }

    async fn is_requested(&self) -> bool {
        self.state.lock().await.first.is_some()
    }

    async fn wait(&self) {
        loop {
            let notified = self.notify.notified();
            if self.state.lock().await.first.is_some() {
                return;
            }
            notified.await;
        }
    }

    pub(crate) async fn outcome(&self) -> ShutdownOutcome {
        let state = self.state.lock().await;
        ShutdownOutcome {
            first: state.first,
            fatal_triggers: state.fatal_triggers.clone(),
        }
    }
}

#[cfg(unix)]
pub(crate) struct SignalReceivers {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl SignalReceivers {
    pub(crate) fn install() -> Result<Self> {
        Ok(Self {
            interrupt: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                .context("unable to register SIGINT handler")?,
            terminate: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .context("unable to register SIGTERM handler")?,
        })
    }
}

#[cfg(not(unix))]
pub(crate) struct SignalReceivers;

#[cfg(not(unix))]
impl SignalReceivers {
    pub(crate) fn install() -> Result<Self> {
        Ok(Self)
    }
}

async fn signal_worker(
    #[cfg(unix)] mut signals: SignalReceivers,
    #[cfg(not(unix))] _signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
) {
    #[cfg(unix)]
    {
        tokio::select! {
            () = coordinator.wait() => {}
            value = signals.interrupt.recv() => {
                if value.is_some() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
            value = signals.terminate.recv() => {
                if value.is_some() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            () = coordinator.wait() => {}
            result = tokio::signal::ctrl_c() => {
                if result.is_ok() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
        }
    }
}

pub(crate) async fn cleanup_worker<S>(
    store: S,
    coordinator: ShutdownCoordinator,
    health: CleanupHealth,
    runtime: debtor_web::state::RuntimeControl,
    interval: Duration,
) where
    S: ExpiredDeletion + Clone,
{
    loop {
        tokio::select! {
            () = coordinator.wait() => return,
            () = tokio::time::sleep(interval) => {
                if !run_session_cleanup_iteration(&store, &health, &runtime, &coordinator).await {
                    return;
                }
            }
        }
    }
}

pub(crate) async fn submission_token_cleanup_worker<S>(
    store: S,
    coordinator: ShutdownCoordinator,
    health: CleanupHealth,
    runtime: debtor_web::state::RuntimeControl,
    interval: Duration,
) where
    S: SubmissionTokenCleanup + Clone,
{
    loop {
        tokio::select! {
            () = coordinator.wait() => return,
            () = tokio::time::sleep(interval) => {
                if !run_submission_token_cleanup_iteration(
                    &store,
                    &health,
                    &runtime,
                    &coordinator,
                )
                .await
                {
                    return;
                }
            }
        }
    }
}

pub(crate) async fn run_session_cleanup_iteration<S>(
    store: &S,
    health: &CleanupHealth,
    runtime: &debtor_web::state::RuntimeControl,
    coordinator: &ShutdownCoordinator,
) -> bool
where
    S: ExpiredDeletion,
{
    if store.delete_expired().await.is_err() {
        report_cleanup_failure(health, runtime, coordinator).await;
        false
    } else {
        true
    }
}

pub(crate) async fn run_submission_token_cleanup_iteration<S>(
    store: &S,
    health: &CleanupHealth,
    runtime: &debtor_web::state::RuntimeControl,
    coordinator: &ShutdownCoordinator,
) -> bool
where
    S: SubmissionTokenCleanup,
{
    if store.cleanup_expired().await.is_err() {
        report_cleanup_failure(health, runtime, coordinator).await;
        false
    } else {
        true
    }
}

async fn supervise_cleanup_worker(
    worker: JoinHandle<()>,
    coordinator: ShutdownCoordinator,
    health: CleanupHealth,
    runtime: debtor_web::state::RuntimeControl,
) {
    let result = worker.await;
    match result {
        Ok(()) if coordinator.is_requested().await => {}
        Ok(()) => {
            report_cleanup_failure(&health, &runtime, &coordinator).await;
            tracing::warn!(
                target: "debtor.runtime",
                event = "cleanup_supervisor_failed",
                category = "worker_exit",
            );
        }
        Err(error) => {
            report_cleanup_failure(&health, &runtime, &coordinator).await;
            let category = if error.is_panic() {
                "worker_panic"
            } else if error.is_cancelled() {
                "worker_cancelled"
            } else {
                "worker_join_failure"
            };
            tracing::warn!(
                target: "debtor.runtime",
                event = "cleanup_supervisor_failed",
                category,
            );
        }
    }
}

async fn report_cleanup_failure(
    health: &CleanupHealth,
    runtime: &debtor_web::state::RuntimeControl,
    coordinator: &ShutdownCoordinator,
) {
    health.mark_unhealthy();
    runtime.close_user_admission();
    coordinator.request(ShutdownTrigger::CleanupFailure).await;
}

struct WalCheckpoint {
    busy: i64,
    log: i64,
    checkpointed: i64,
}

pub(crate) async fn checkpoint_pool(pool: &SqlitePool) -> bool {
    checkpoint_pool_with_timeout(pool, WAL_CHECKPOINT_TIMEOUT).await
}

pub(crate) async fn checkpoint_pool_with_timeout(pool: &SqlitePool, timeout: Duration) -> bool {
    // SQLite exposes wal_checkpoint output without declared column types.
    // Keep this static pragma checked for syntax while decoding its fixed shape explicitly.
    tokio::time::timeout(
        timeout,
        sqlx::query_as_unchecked!(WalCheckpoint, "PRAGMA wal_checkpoint(TRUNCATE)").fetch_one(pool),
    )
    .await
    .is_ok_and(|result| {
        result.is_ok_and(|checkpoint| {
            let _ = (checkpoint.log, checkpoint.checkpointed);
            checkpoint.busy == 0
        })
    })
}

pub(crate) async fn close_pool(pool: &SqlitePool) -> bool {
    close_pool_with_timeout(pool, POOL_CLOSE_TIMEOUT).await
}

pub(crate) async fn close_pool_with_timeout(pool: &SqlitePool, timeout: Duration) -> bool {
    tokio::time::timeout(timeout, pool.close()).await.is_ok()
}

pub(crate) async fn run_runtime_with_options(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
    cleanup_interval: Duration,
) -> Result<()> {
    run_runtime_with_timeouts(
        runtime,
        listener,
        signals,
        coordinator,
        cleanup_interval,
        HTTP_DRAIN_TIMEOUT,
    )
    .await
}

pub(crate) async fn run_runtime_with_coordinator(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
) -> Result<()> {
    run_runtime_with_options(runtime, listener, signals, coordinator, CLEANUP_INTERVAL).await
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn run_runtime_with_timeouts(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
    cleanup_interval: Duration,
    http_drain_timeout: Duration,
) -> Result<()> {
    run_runtime_with_storage_timeouts(
        runtime,
        listener,
        signals,
        coordinator,
        cleanup_interval,
        http_drain_timeout,
        WAL_CHECKPOINT_TIMEOUT,
        POOL_CLOSE_TIMEOUT,
    )
    .await
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) async fn run_runtime_with_storage_timeouts(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
    cleanup_interval: Duration,
    http_drain_timeout: Duration,
    checkpoint_timeout: Duration,
    pool_close_timeout: Duration,
) -> Result<()> {
    let cleanup_handle: JoinHandle<()> = tokio::spawn(cleanup_worker(
        runtime.session_store.clone(),
        coordinator.clone(),
        runtime.cleanup_health.clone(),
        runtime.runtime.clone(),
        cleanup_interval,
    ));
    let submission_token_cleanup_handle: JoinHandle<()> =
        tokio::spawn(submission_token_cleanup_worker(
            runtime.submission_token_store.clone(),
            coordinator.clone(),
            runtime.cleanup_health.clone(),
            runtime.runtime.clone(),
            cleanup_interval,
        ));
    let cleanup_abort = cleanup_handle.abort_handle();
    let submission_token_cleanup_abort = submission_token_cleanup_handle.abort_handle();
    let mut cleanup_supervisor = tokio::spawn(supervise_cleanup_worker(
        cleanup_handle,
        coordinator.clone(),
        runtime.cleanup_health.clone(),
        runtime.runtime.clone(),
    ));
    let mut submission_token_cleanup_supervisor = tokio::spawn(supervise_cleanup_worker(
        submission_token_cleanup_handle,
        coordinator.clone(),
        runtime.cleanup_health.clone(),
        runtime.runtime.clone(),
    ));
    let mut signal_handle: JoinHandle<()> =
        tokio::spawn(signal_worker(signals, coordinator.clone()));
    let server_shutdown = Arc::new(Notify::new());
    let server_shutdown_signal = server_shutdown.clone();
    let mut server = Box::pin(
        axum::serve(
            listener,
            runtime
                .app
                .layer(axum::middleware::from_fn({
                    let runtime_control = runtime.runtime.clone();
                    move |request, next| {
                        debtor_web::middleware::force_safe_request_shutdown(
                            runtime_control.clone(),
                            request,
                            next,
                        )
                    }
                }))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { server_shutdown_signal.notified().await })
        .into_future(),
    );

    let server_finished = await_server_or_shutdown(&mut server, &coordinator).await;
    runtime.runtime.close_user_admission();
    runtime.mutations.close();

    if !server_finished {
        let server_shutdown_signal = server_shutdown.clone();
        let drain_deadline = async move {
            tokio::time::sleep(http_drain_timeout).await;
            server_shutdown_signal.notify_one();
        };
        tokio::pin!(drain_deadline);
        tokio::select! {
            result = &mut server => {
                if result.is_err() {
                    coordinator.request(ShutdownTrigger::HttpFailure).await;
                }
            }
            () = &mut drain_deadline => {
                runtime.runtime.force_safe_request_shutdown();
                server_shutdown.notify_one();
            }
        }
    }
    drop(server);
    runtime.mutations.wait_until_empty().await;

    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut cleanup_supervisor).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => {
            report_cleanup_failure(&runtime.cleanup_health, &runtime.runtime, &coordinator).await;
        }
        Err(_) => {
            cleanup_abort.abort();
            cleanup_supervisor.abort();
            let _ = cleanup_supervisor.await;
            report_cleanup_failure(&runtime.cleanup_health, &runtime.runtime, &coordinator).await;
        }
    }
    match tokio::time::timeout(
        CLEANUP_STOP_TIMEOUT,
        &mut submission_token_cleanup_supervisor,
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(_)) => {
            report_cleanup_failure(&runtime.cleanup_health, &runtime.runtime, &coordinator).await;
        }
        Err(_) => {
            submission_token_cleanup_abort.abort();
            submission_token_cleanup_supervisor.abort();
            let _ = submission_token_cleanup_supervisor.await;
            report_cleanup_failure(&runtime.cleanup_health, &runtime.runtime, &coordinator).await;
        }
    }
    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut signal_handle).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => coordinator.request(ShutdownTrigger::SignalFailure).await,
        Err(_) => {
            signal_handle.abort();
            let _ = signal_handle.await;
            coordinator.request(ShutdownTrigger::SignalFailure).await;
        }
    }

    #[cfg(test)]
    runtime.shutdown_events.mark_checkpoint_started();
    let checkpointed = if checkpoint_timeout == WAL_CHECKPOINT_TIMEOUT {
        checkpoint_pool(&runtime.pool).await
    } else {
        checkpoint_pool_with_timeout(&runtime.pool, checkpoint_timeout).await
    };
    if !checkpointed {
        coordinator
            .request(ShutdownTrigger::CheckpointFailure)
            .await;
    }
    let pool_closed = if pool_close_timeout == POOL_CLOSE_TIMEOUT {
        close_pool(&runtime.pool).await
    } else {
        close_pool_with_timeout(&runtime.pool, pool_close_timeout).await
    };
    if !pool_closed {
        coordinator.request(ShutdownTrigger::PoolCloseFailure).await;
    }

    let outcome = coordinator.outcome().await;
    let success = outcome.first.is_some() && outcome.fatal_triggers.is_empty();
    let trigger = outcome.first.map_or("unknown", ShutdownTrigger::name);
    if success {
        tracing::info!(
            target: "debtor.runtime",
            event = "shutdown_complete",
            success,
            trigger,
            fatal_failure_count = outcome.fatal_triggers.len(),
        );
    } else {
        tracing::warn!(
            target: "debtor.runtime",
            event = "shutdown_complete",
            success,
            trigger,
            fatal_failure_count = outcome.fatal_triggers.len(),
        );
    }
    if !success {
        return Err(anyhow!("runtime shutdown failed"));
    }
    Ok(())
}

pub(crate) async fn await_server_or_shutdown<F>(
    server: &mut F,
    coordinator: &ShutdownCoordinator,
) -> bool
where
    F: Future<Output = std::io::Result<()>> + Unpin,
{
    tokio::select! {
        result = server => {
            if result.is_err() || coordinator.outcome().await.first.is_none() {
                coordinator.request(ShutdownTrigger::HttpFailure).await;
            }
            true
        }
        () = coordinator.wait() => false,
    }
}

#[cfg(test)]
pub(crate) async fn drain_result<F>(future: F, timeout: Duration) -> Option<F::Output>
where
    F: Future,
{
    tokio::time::timeout(timeout, future).await.ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unexpected_cleanup_exit_fails_readiness_and_closes_admission_once() {
        let coordinator = ShutdownCoordinator::default();
        let health = CleanupHealth::new();
        let runtime = debtor_web::state::RuntimeControl::default();
        let first = tokio::spawn(async { panic!("unexpected cleanup exit") });
        let second = tokio::spawn(async { panic!("unexpected cleanup exit") });

        tokio::join!(
            supervise_cleanup_worker(first, coordinator.clone(), health.clone(), runtime.clone(),),
            supervise_cleanup_worker(second, coordinator.clone(), health.clone(), runtime.clone(),),
        );

        assert!(!health.is_healthy());
        assert!(!runtime.user_admission_open());
        let outcome = coordinator.outcome().await;
        assert_eq!(outcome.first, Some(ShutdownTrigger::CleanupFailure));
        assert_eq!(
            outcome.fatal_triggers,
            vec![ShutdownTrigger::CleanupFailure]
        );
    }

    #[tokio::test]
    async fn cancelled_cleanup_exit_is_supervisor_failure() {
        let coordinator = ShutdownCoordinator::default();
        let health = CleanupHealth::new();
        let runtime = debtor_web::state::RuntimeControl::default();
        let worker = tokio::spawn(std::future::pending::<()>());
        worker.abort();

        supervise_cleanup_worker(worker, coordinator.clone(), health.clone(), runtime.clone())
            .await;

        assert!(!health.is_healthy());
        assert!(!runtime.user_admission_open());
        assert_eq!(
            coordinator.outcome().await.first,
            Some(ShutdownTrigger::CleanupFailure)
        );
    }

    #[tokio::test]
    async fn worker_exit_after_coordinated_shutdown_is_not_a_failure() {
        let coordinator = ShutdownCoordinator::default();
        let health = CleanupHealth::new();
        let runtime = debtor_web::state::RuntimeControl::default();
        coordinator.request(ShutdownTrigger::Signal).await;
        let worker = tokio::spawn(async {});

        supervise_cleanup_worker(worker, coordinator, health.clone(), runtime.clone()).await;

        assert!(health.is_healthy());
        assert!(runtime.user_admission_open());
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn dispatched_mutation_registry_closes_and_waits_for_active_leases() {
        let registry = DispatchedMutationRegistry::default();
        let lease = registry.try_register().expect("registry accepts dispatch");

        registry.close();
        assert!(registry.try_register().is_none());

        let mut wait = tokio::spawn({
            let registry = registry.clone();
            async move {
                registry.wait_until_empty().await;
            }
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut wait)
                .await
                .is_err()
        );

        drop(lease);
        wait.await.expect("empty registry barrier");
    }
}
