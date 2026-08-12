use std::future::Future;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
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

    fn mark_unhealthy(&self) {
        self.0.store(false, Ordering::Release);
    }
}

impl SupervisorReadiness for CleanupHealth {
    fn is_healthy(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShutdownTrigger {
    Signal,
    SignalFailure,
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
        let mut state = self.state.lock().await;
        let first = state.first.is_none();
        if first {
            state.first = Some(trigger);
        }
        if trigger.is_fatal() {
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
        } else if trigger.is_fatal() {
            tracing::warn!(
                target: "debtor.runtime",
                event = "shutdown_failure",
                trigger = trigger.name(),
            );
        }
        self.notify.notify_waiters();
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
    interval: Duration,
) where
    S: ExpiredDeletion + Clone,
{
    loop {
        tokio::select! {
            () = coordinator.wait() => return,
            () = tokio::time::sleep(interval) => {
                if store.delete_expired().await.is_err() {
                    health.mark_unhealthy();
                    coordinator.request(ShutdownTrigger::CleanupFailure).await;
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
    interval: Duration,
) where
    S: SubmissionTokenCleanup + Clone,
{
    loop {
        tokio::select! {
            () = coordinator.wait() => return,
            () = tokio::time::sleep(interval) => {
                if store.cleanup_expired().await.is_err() {
                    health.mark_unhealthy();
                    coordinator.request(ShutdownTrigger::CleanupFailure).await;
                    return;
                }
            }
        }
    }
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

pub(crate) async fn run_runtime(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
) -> Result<()> {
    run_runtime_with_options(
        runtime,
        listener,
        signals,
        ShutdownCoordinator::default(),
        CLEANUP_INTERVAL,
    )
    .await
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

pub(crate) async fn run_runtime_with_timeouts(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
    cleanup_interval: Duration,
    http_drain_timeout: Duration,
) -> Result<()> {
    let mut cleanup_handle: JoinHandle<()> = tokio::spawn(cleanup_worker(
        runtime.session_store.clone(),
        coordinator.clone(),
        runtime.cleanup_health.clone(),
        cleanup_interval,
    ));
    let mut submission_token_cleanup_handle: JoinHandle<()> =
        tokio::spawn(submission_token_cleanup_worker(
            runtime.submission_token_store.clone(),
            coordinator.clone(),
            runtime.cleanup_health.clone(),
            cleanup_interval,
        ));
    let mut signal_handle: JoinHandle<()> =
        tokio::spawn(signal_worker(signals, coordinator.clone()));
    let server_shutdown = coordinator.clone();
    let mut server = Box::pin(
        axum::serve(
            listener,
            runtime
                .app
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { server_shutdown.wait().await })
        .into_future(),
    );

    let server_finished = await_server_or_shutdown(&mut server, &coordinator).await;

    if !server_finished
        && let Some(result) = drain_result(&mut server, http_drain_timeout).await
        && result.is_err()
    {
        coordinator.request(ShutdownTrigger::HttpFailure).await;
    }
    drop(server);

    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut cleanup_handle).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => coordinator.request(ShutdownTrigger::CleanupFailure).await,
        Err(_) => {
            cleanup_handle.abort();
            let _ = cleanup_handle.await;
            coordinator.request(ShutdownTrigger::CleanupFailure).await;
        }
    }
    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut submission_token_cleanup_handle).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => coordinator.request(ShutdownTrigger::CleanupFailure).await,
        Err(_) => {
            submission_token_cleanup_handle.abort();
            let _ = submission_token_cleanup_handle.await;
            coordinator.request(ShutdownTrigger::CleanupFailure).await;
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

    if !checkpoint_pool(&runtime.pool).await {
        coordinator
            .request(ShutdownTrigger::CheckpointFailure)
            .await;
    }
    if !close_pool(&runtime.pool).await {
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

pub(crate) async fn drain_result<F>(future: F, timeout: Duration) -> Option<F::Output>
where
    F: Future,
{
    tokio::time::timeout(timeout, future).await.ok()
}
