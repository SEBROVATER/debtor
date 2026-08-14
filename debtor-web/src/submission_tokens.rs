//! Bounded process-local submission-token storage.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    sync::Arc,
};

use async_trait::async_trait;
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Maximum number of live anonymous Login submission tokens.
pub const ANONYMOUS_CAPACITY: usize = 4_096;
/// Maximum number of live authenticated page-scoped submission tokens.
pub const AUTHENTICATED_CAPACITY: usize = 1_024;
/// Maximum number of live authenticated tokens for one session.
pub const AUTHENTICATED_SESSION_CAPACITY: usize = 32;

const ANONYMOUS_LIFETIME: Duration = Duration::minutes(10);
const AUTHENTICATED_LIFETIME: Duration = Duration::minutes(30);
const ANONYMOUS_CAPACITY_ERROR: &str = "anonymous submission capacity reached";
const AUTHENTICATED_CAPACITY_ERROR: &str = "authenticated submission capacity reached";
const CONFLICT_ERROR: &str = "invalid submission token";

/// Identifies the isolated token pool used by a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenPool {
    /// Anonymous Login form tokens.
    Anonymous,
    /// Authenticated page-scoped form tokens.
    Authenticated,
}

#[derive(Debug, Clone)]
struct TokenRecord {
    session_id: i128,
    expiry: OffsetDateTime,
    reserved: bool,
}

#[derive(Debug, Default)]
struct PoolState {
    records: HashMap<String, TokenRecord>,
    session_tokens: HashMap<i128, BTreeSet<String>>,
    expiry_index: BTreeMap<OffsetDateTime, BTreeSet<String>>,
}

#[derive(Debug, Default)]
struct StoreState {
    anonymous: PoolState,
    authenticated: PoolState,
}

/// A bounded, process-local owner for all Login and authenticated form tokens.
pub struct SubmissionTokenStore {
    state: Arc<Mutex<StoreState>>,
    now: Arc<dyn Fn() -> OffsetDateTime + Send + Sync>,
}

impl Clone for SubmissionTokenStore {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            now: self.now.clone(),
        }
    }
}

impl fmt::Debug for SubmissionTokenStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SubmissionTokenStore")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for SubmissionTokenStore {
    fn default() -> Self {
        Self::with_clock(OffsetDateTime::now_utc)
    }
}

impl SubmissionTokenStore {
    /// Creates a store using the supplied clock.
    pub fn with_clock<F>(clock: F) -> Self
    where
        F: Fn() -> OffsetDateTime + Send + Sync + 'static,
    {
        Self {
            state: Arc::new(Mutex::new(StoreState::default())),
            now: Arc::new(clock),
        }
    }

    fn current_time(&self) -> OffsetDateTime {
        (self.now)()
    }

    fn pool_mut(state: &mut StoreState, pool: TokenPool) -> &mut PoolState {
        match pool {
            TokenPool::Anonymous => &mut state.anonymous,
            TokenPool::Authenticated => &mut state.authenticated,
        }
    }

    fn lifetime(pool: TokenPool) -> Duration {
        match pool {
            TokenPool::Anonymous => ANONYMOUS_LIFETIME,
            TokenPool::Authenticated => AUTHENTICATED_LIFETIME,
        }
    }

    fn capacity(pool: TokenPool) -> usize {
        match pool {
            TokenPool::Anonymous => ANONYMOUS_CAPACITY,
            TokenPool::Authenticated => AUTHENTICATED_CAPACITY,
        }
    }

    fn remove_from_expiry_index(pool: &mut PoolState, token: &str, expiry: OffsetDateTime) {
        let remove_bucket = pool.expiry_index.get_mut(&expiry).is_some_and(|tokens| {
            tokens.remove(token);
            tokens.is_empty()
        });
        if remove_bucket {
            pool.expiry_index.remove(&expiry);
        }
    }

    fn remove_token(pool: &mut PoolState, token: &str) -> Option<TokenRecord> {
        let record = pool.records.remove(token)?;
        Self::remove_from_expiry_index(pool, token, record.expiry);
        if let Some(tokens) = pool.session_tokens.get_mut(&record.session_id) {
            tokens.remove(token);
            if tokens.is_empty() {
                pool.session_tokens.remove(&record.session_id);
            }
        }
        Some(record)
    }

    fn remove_expired(pool: &mut PoolState, now: OffsetDateTime) {
        while let Some((&expiry, _)) = pool.expiry_index.first_key_value() {
            if expiry > now {
                break;
            }
            let tokens = pool.expiry_index.remove(&expiry).unwrap_or_default();
            for token in tokens {
                if let Some(record) = pool.records.remove(&token)
                    && let Some(session_tokens) = pool.session_tokens.get_mut(&record.session_id)
                {
                    session_tokens.remove(&token);
                    if session_tokens.is_empty() {
                        pool.session_tokens.remove(&record.session_id);
                    }
                }
            }
        }
    }

    fn cleanup_pool(state: &mut StoreState, pool: TokenPool, now: OffsetDateTime) {
        Self::remove_expired(Self::pool_mut(state, pool), now);
    }

    fn insert_token(pool: &mut PoolState, session_id: i128, expiry: OffsetDateTime) -> String {
        let token = loop {
            let token = Uuid::new_v4().to_string();
            if !pool.records.contains_key(&token) {
                break token;
            }
        };
        pool.records.insert(
            token.clone(),
            TokenRecord {
                session_id,
                expiry,
                reserved: false,
            },
        );
        pool.session_tokens
            .entry(session_id)
            .or_default()
            .insert(token.clone());
        pool.expiry_index
            .entry(expiry)
            .or_default()
            .insert(token.clone());
        token
    }

    /// Issues or refreshes the one anonymous Login token for a session.
    ///
    /// # Errors
    ///
    /// Returns `Capacity` when the anonymous pool cannot admit a new token.
    pub async fn issue(
        &self,
        session_id: tower_sessions::session::Id,
    ) -> Result<String, IssueError> {
        let session_id = session_id.0;
        let now = self.current_time();
        let expiry = now
            .checked_add(Self::lifetime(TokenPool::Anonymous))
            .ok_or(IssueError::ClockRange)?;
        let mut state = self.state.lock().await;
        Self::cleanup_pool(&mut state, TokenPool::Anonymous, now);
        let pool = &mut state.anonymous;

        if let Some(token) = pool
            .session_tokens
            .get(&session_id)
            .and_then(|tokens| tokens.iter().next())
            .cloned()
        {
            if let Some(record) = pool.records.get(&token)
                && !record.reserved
            {
                let previous_expiry = record.expiry;
                Self::remove_from_expiry_index(pool, &token, previous_expiry);
                if let Some(record) = pool.records.get_mut(&token) {
                    record.expiry = expiry;
                }
                pool.expiry_index
                    .entry(expiry)
                    .or_default()
                    .insert(token.clone());
                return Ok(token);
            }
            Self::remove_token(pool, &token);
        }

        if pool.records.len() >= ANONYMOUS_CAPACITY {
            return Err(IssueError::Capacity(TokenPool::Anonymous));
        }
        Ok(Self::insert_token(pool, session_id, expiry))
    }

    /// Issues a fresh page-scoped token for an authenticated response.
    ///
    /// # Errors
    ///
    /// Returns `Capacity` when the global or per-session authenticated bound is full.
    pub async fn issue_authenticated(
        &self,
        session_id: tower_sessions::session::Id,
    ) -> Result<String, IssueError> {
        let session_id = session_id.0;
        let now = self.current_time();
        let expiry = now
            .checked_add(Self::lifetime(TokenPool::Authenticated))
            .ok_or(IssueError::ClockRange)?;
        let mut state = self.state.lock().await;
        Self::cleanup_pool(&mut state, TokenPool::Authenticated, now);
        let pool = &mut state.authenticated;
        if pool.records.len() >= Self::capacity(TokenPool::Authenticated)
            || pool
                .session_tokens
                .get(&session_id)
                .is_some_and(|tokens| tokens.len() >= AUTHENTICATED_SESSION_CAPACITY)
        {
            return Err(IssueError::Capacity(TokenPool::Authenticated));
        }
        Ok(Self::insert_token(pool, session_id, expiry))
    }

    /// Validates a token without reserving it for dispatch.
    ///
    /// # Errors
    ///
    /// Returns `Conflict` for absent, expired, reserved, or session-mismatched tokens.
    pub(crate) async fn validate(
        &self,
        session_id: tower_sessions::session::Id,
        pool: TokenPool,
        token: &str,
    ) -> Result<(), ReserveError> {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::cleanup_pool(&mut state, pool, now);
        let pool = Self::pool_mut(&mut state, pool);
        if pool.records.get(token).is_some_and(|record| {
            record.session_id == session_id.0 && !record.reserved && record.expiry > now
        }) {
            Ok(())
        } else {
            Err(ReserveError::Conflict)
        }
    }

    /// Validates, marks dispatch, and terminally reserves a token as one atomic boundary.
    ///
    /// The callback runs while the store lock is held. It must only mark the request's
    /// dispatch boundary and must not await or perform external work. A rejected callback
    /// leaves the token available for retry.
    ///
    /// # Errors
    ///
    /// Returns `Conflict` for invalid tokens and `Deadline` when dispatch cannot cross the
    /// request's pre-dispatch boundary.
    pub(crate) async fn reserve_and_dispatch(
        &self,
        session_id: tower_sessions::session::Id,
        pool_kind: TokenPool,
        token: &str,
        dispatch: impl FnOnce() -> Result<(), ()>,
    ) -> Result<(), ReserveError> {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::cleanup_pool(&mut state, pool_kind, now);
        let pool = Self::pool_mut(&mut state, pool_kind);
        let valid_record = pool.records.get(token).is_some_and(|record| {
            record.session_id == session_id.0 && !record.reserved && record.expiry > now
        });
        if !valid_record {
            return Err(ReserveError::Conflict);
        }
        dispatch().map_err(|()| ReserveError::Deadline)?;
        pool.records
            .get_mut(token)
            .ok_or(ReserveError::Conflict)?
            .reserved = true;
        Ok(())
    }

    /// Reserves a token and crosses the request dispatch boundary atomically.
    pub(crate) async fn reserve_form_and_dispatch(
        &self,
        session_id: tower_sessions::session::Id,
        pool: TokenPool,
        token: &str,
        dispatch: impl FnOnce() -> Result<(), ()>,
    ) -> Result<(), ReserveError> {
        self.reserve_and_dispatch(session_id, pool, token, dispatch)
            .await
    }

    /// Removes all authenticated tokens owned by a flushed session.
    pub(crate) async fn remove_authenticated_session(
        &self,
        session_id: tower_sessions::session::Id,
    ) {
        let mut state = self.state.lock().await;
        let tokens = state
            .authenticated
            .session_tokens
            .get(&session_id.0)
            .map(|tokens| tokens.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for token in tokens {
            Self::remove_token(&mut state.authenticated, &token);
        }
    }

    #[cfg(test)]
    async fn len(&self, pool: TokenPool) -> usize {
        let state = self.state.lock().await;
        match pool {
            TokenPool::Anonymous => state.anonymous.records.len(),
            TokenPool::Authenticated => state.authenticated.records.len(),
        }
    }
}

/// Submission-token issuance failed because a bounded pool is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueError {
    /// No new token can be admitted without evicting a live token.
    Capacity(TokenPool),
    /// The configured clock cannot represent the token expiry.
    ClockRange,
}

/// A token could not cross the future dispatch boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveError {
    /// The token is absent, expired, reserved, or session-mismatched.
    Conflict,
    /// The request's pre-dispatch deadline elapsed.
    Deadline,
}

impl fmt::Display for IssueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Capacity(TokenPool::Anonymous) => ANONYMOUS_CAPACITY_ERROR,
            Self::Capacity(TokenPool::Authenticated) => AUTHENTICATED_CAPACITY_ERROR,
            Self::ClockRange => "submission expiry unavailable",
        })
    }
}

impl fmt::Display for ReserveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Conflict => CONFLICT_ERROR,
            Self::Deadline => "request timed out",
        })
    }
}

impl SubmissionTokenStore {
    /// Removes expired tokens using indexed expiry buckets.
    pub async fn delete_expired(&self) {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::remove_expired(&mut state.anonymous, now);
        Self::remove_expired(&mut state.authenticated, now);
    }
}

/// Cleanup contract used by the root supervisor.
#[async_trait]
pub trait SubmissionTokenCleanup: Send + Sync {
    /// Removes expired token records.
    async fn cleanup_expired(&self) -> Result<(), ()>;
}

#[async_trait]
impl SubmissionTokenCleanup for SubmissionTokenStore {
    async fn cleanup_expired(&self) -> Result<(), ()> {
        self.delete_expired().await;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use time::OffsetDateTime;
    use tower_sessions::session::Id;

    use super::{
        ANONYMOUS_CAPACITY, AUTHENTICATED_CAPACITY, AUTHENTICATED_SESSION_CAPACITY, IssueError,
        ReserveError, SubmissionTokenStore, TokenPool,
    };

    fn store() -> (Arc<Mutex<OffsetDateTime>>, SubmissionTokenStore) {
        let now = Arc::new(Mutex::new(OffsetDateTime::UNIX_EPOCH));
        let source = now.clone();
        let store = SubmissionTokenStore::with_clock(move || *source.lock().unwrap());
        (now, store)
    }

    fn id(value: i128) -> Id {
        Id(value)
    }

    #[tokio::test]
    async fn issues_one_anonymous_token_per_session_and_refreshes_expiry() {
        let (now, store) = store();
        let first = store.issue(id(1)).await.expect("first token");
        *now.lock().unwrap() += time::Duration::minutes(5);
        let second = store.issue(id(1)).await.expect("refreshed token");
        assert_eq!(first, second);
        assert_eq!(store.len(TokenPool::Anonymous).await, 1);
        *now.lock().unwrap() += time::Duration::minutes(11);
        store.delete_expired().await;
        assert_eq!(store.len(TokenPool::Anonymous).await, 0);
    }

    #[tokio::test]
    async fn anonymous_capacity_is_exact_and_does_not_evict_existing_tokens() {
        let (_, store) = store();
        let mut first = String::new();
        for session in 0..ANONYMOUS_CAPACITY {
            let token = store
                .issue(id(session as i128))
                .await
                .expect("capacity token");
            if session == 0 {
                first = token;
            }
        }
        assert_eq!(
            store.issue(id(ANONYMOUS_CAPACITY as i128)).await,
            Err(IssueError::Capacity(TokenPool::Anonymous))
        );
        assert_eq!(store.issue(id(0)).await.expect("existing token"), first);
    }

    #[tokio::test]
    async fn authenticated_pages_get_unique_tokens_with_a_per_session_bound() {
        let (_, store) = store();
        let first = store.issue_authenticated(id(1)).await.expect("first token");
        for _ in 1..AUTHENTICATED_SESSION_CAPACITY {
            store
                .issue_authenticated(id(1))
                .await
                .expect("session token");
        }
        assert_eq!(
            store.len(TokenPool::Authenticated).await,
            AUTHENTICATED_SESSION_CAPACITY
        );
        assert_eq!(
            store.issue_authenticated(id(1)).await,
            Err(IssueError::Capacity(TokenPool::Authenticated))
        );
        let second = store
            .issue_authenticated(id(2))
            .await
            .expect("second session token");
        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn authenticated_global_capacity_is_bounded_without_eviction() {
        let (_, store) = store();
        for session in 0..(AUTHENTICATED_CAPACITY / AUTHENTICATED_SESSION_CAPACITY) {
            for _ in 0..AUTHENTICATED_SESSION_CAPACITY {
                store
                    .issue_authenticated(id(session as i128))
                    .await
                    .expect("capacity token");
            }
        }
        assert_eq!(
            store.len(TokenPool::Authenticated).await,
            AUTHENTICATED_CAPACITY
        );
        assert_eq!(
            store.issue_authenticated(id(99_999)).await,
            Err(IssueError::Capacity(TokenPool::Authenticated))
        );
    }

    #[tokio::test]
    async fn pools_are_isolated() {
        let (_, store) = store();
        for session in 0..ANONYMOUS_CAPACITY {
            store
                .issue(id(session as i128))
                .await
                .expect("anonymous capacity");
        }
        for session in 0..AUTHENTICATED_SESSION_CAPACITY {
            store
                .issue_authenticated(id(session as i128))
                .await
                .expect("authenticated capacity");
        }
        assert_eq!(store.len(TokenPool::Anonymous).await, ANONYMOUS_CAPACITY);
        assert_eq!(
            store.len(TokenPool::Authenticated).await,
            AUTHENTICATED_SESSION_CAPACITY
        );
    }

    #[tokio::test]
    async fn validation_is_session_bound_and_does_not_reserve() {
        let (_, store) = store();
        let token = store.issue_authenticated(id(1)).await.expect("token");
        assert_eq!(
            store
                .validate(id(2), TokenPool::Authenticated, &token)
                .await,
            Err(ReserveError::Conflict)
        );
        store
            .validate(id(1), TokenPool::Authenticated, &token)
            .await
            .expect("validate token");
        store
            .validate(id(1), TokenPool::Authenticated, &token)
            .await
            .expect("validate again");
    }

    #[tokio::test]
    async fn dispatch_rejection_does_not_consume_and_success_is_terminal() {
        let (_, store) = store();
        let token = store.issue_authenticated(id(1)).await.expect("token");
        assert_eq!(
            store
                .reserve_and_dispatch(id(1), TokenPool::Authenticated, &token, || Err(()))
                .await,
            Err(ReserveError::Deadline)
        );
        store
            .reserve_and_dispatch(id(1), TokenPool::Authenticated, &token, || Ok(()))
            .await
            .expect("dispatch token");
        assert_eq!(
            store
                .validate(id(1), TokenPool::Authenticated, &token)
                .await,
            Err(ReserveError::Conflict)
        );
    }

    #[tokio::test]
    async fn concurrent_authenticated_reservation_has_one_winner() {
        let (_, store) = store();
        let token = store.issue_authenticated(id(1)).await.expect("token");
        let barrier = Arc::new(tokio::sync::Barrier::new(3));
        let first_store = store.clone();
        let second_store = store.clone();
        let first_barrier = barrier.clone();
        let second_barrier = barrier.clone();
        let first_token = token.clone();
        let second_token = token;
        let first = tokio::spawn(async move {
            first_barrier.wait().await;
            first_store
                .reserve_and_dispatch(id(1), TokenPool::Authenticated, &first_token, || Ok(()))
                .await
        });
        let second = tokio::spawn(async move {
            second_barrier.wait().await;
            second_store
                .reserve_and_dispatch(id(1), TokenPool::Authenticated, &second_token, || Ok(()))
                .await
        });
        barrier.wait().await;
        let results = [
            first.await.expect("first reservation"),
            second.await.expect("second reservation"),
        ];
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| **result == Err(ReserveError::Conflict))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn authenticated_tokens_expire_after_thirty_minutes() {
        let (now, store) = store();
        let token = store.issue_authenticated(id(1)).await.expect("token");
        *now.lock().unwrap() += time::Duration::minutes(30) + time::Duration::seconds(1);
        store.delete_expired().await;
        assert_eq!(
            store
                .validate(id(1), TokenPool::Authenticated, &token)
                .await,
            Err(ReserveError::Conflict)
        );
        assert_eq!(store.len(TokenPool::Authenticated).await, 0);
    }

    #[tokio::test]
    async fn flushing_a_session_removes_all_authenticated_tokens() {
        let (_, store) = store();
        for _ in 0..AUTHENTICATED_SESSION_CAPACITY {
            store
                .issue_authenticated(id(1))
                .await
                .expect("session token");
        }
        store.remove_authenticated_session(id(1)).await;
        assert_eq!(store.len(TokenPool::Authenticated).await, 0);
        store
            .issue_authenticated(id(1))
            .await
            .expect("reused capacity");
    }

    #[tokio::test]
    async fn expired_tokens_are_physically_removed_by_indexed_cleanup() {
        let (now, store) = store();
        store.issue_authenticated(id(1)).await.expect("token");
        store.issue(id(2)).await.expect("token");
        *now.lock().unwrap() += Duration::from_secs(1_801);
        store.delete_expired().await;
        assert_eq!(store.len(TokenPool::Authenticated).await, 0);
        assert_eq!(store.len(TokenPool::Anonymous).await, 0);
    }
}
