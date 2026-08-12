//! Bounded anonymous submission-token storage.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    sync::Arc,
};

use async_trait::async_trait;
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Maximum number of live anonymous submission tokens.
pub const ANONYMOUS_CAPACITY: usize = 4_096;

const TOKEN_LIFETIME: Duration = Duration::minutes(10);
const CAPACITY_ERROR: &str = "anonymous submission capacity reached";
const CONFLICT_ERROR: &str = "invalid submission token";

#[derive(Debug, Clone)]
struct TokenRecord {
    session_id: i128,
    expiry: OffsetDateTime,
    reserved: bool,
}

#[derive(Debug, Default)]
struct StoreState {
    records: HashMap<String, TokenRecord>,
    session_tokens: HashMap<i128, String>,
    expiry_index: BTreeMap<OffsetDateTime, BTreeSet<String>>,
}

/// A bounded, process-local store for anonymous Login submission tokens.
pub struct AnonymousSubmissionTokenStore {
    state: Arc<Mutex<StoreState>>,
    now: Arc<dyn Fn() -> OffsetDateTime + Send + Sync>,
}

impl Clone for AnonymousSubmissionTokenStore {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            now: self.now.clone(),
        }
    }
}

impl fmt::Debug for AnonymousSubmissionTokenStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AnonymousSubmissionTokenStore")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for AnonymousSubmissionTokenStore {
    fn default() -> Self {
        Self::with_clock(OffsetDateTime::now_utc)
    }
}

impl AnonymousSubmissionTokenStore {
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

    fn remove_from_index(state: &mut StoreState, token: &str, expiry: OffsetDateTime) {
        let remove_bucket = state.expiry_index.get_mut(&expiry).is_some_and(|tokens| {
            tokens.remove(token);
            tokens.is_empty()
        });
        if remove_bucket {
            state.expiry_index.remove(&expiry);
        }
    }

    fn remove_token(state: &mut StoreState, token: &str) -> Option<TokenRecord> {
        let record = state.records.remove(token)?;
        Self::remove_from_index(state, token, record.expiry);
        if state
            .session_tokens
            .get(&record.session_id)
            .is_some_and(|value| value == token)
        {
            state.session_tokens.remove(&record.session_id);
        }
        Some(record)
    }

    fn remove_expired(state: &mut StoreState, now: OffsetDateTime) {
        while let Some((&expiry, _)) = state.expiry_index.first_key_value() {
            if expiry > now {
                break;
            }
            let tokens = state.expiry_index.remove(&expiry).unwrap_or_default();
            for token in tokens {
                if let Some(record) = state.records.remove(&token)
                    && state
                        .session_tokens
                        .get(&record.session_id)
                        .is_some_and(|value| value == &token)
                {
                    state.session_tokens.remove(&record.session_id);
                }
            }
        }
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
        let expiry = now + TOKEN_LIFETIME;
        let mut state = self.state.lock().await;
        Self::remove_expired(&mut state, now);

        if let Some(token) = state.session_tokens.get(&session_id).cloned() {
            if let Some(record) = state.records.get(&token)
                && !record.reserved
            {
                let previous_expiry = record.expiry;
                Self::remove_from_index(&mut state, &token, previous_expiry);
                if let Some(record) = state.records.get_mut(&token) {
                    record.expiry = expiry;
                }
                state
                    .expiry_index
                    .entry(expiry)
                    .or_default()
                    .insert(token.clone());
                return Ok(token);
            }
            Self::remove_token(&mut state, &token);
        }

        if state.records.len() >= ANONYMOUS_CAPACITY {
            return Err(IssueError::Capacity);
        }

        let token = Uuid::new_v4().to_string();
        state.records.insert(
            token.clone(),
            TokenRecord {
                session_id,
                expiry,
                reserved: false,
            },
        );
        state.session_tokens.insert(session_id, token.clone());
        state
            .expiry_index
            .entry(expiry)
            .or_default()
            .insert(token.clone());
        Ok(token)
    }

    /// Reserves a token for the Login dispatch boundary.
    ///
    /// This is intentionally not called by Story 1.4. Story 1.5 uses it
    /// immediately before password verification and keeps reservation terminal.
    ///
    /// # Errors
    ///
    /// Returns `Conflict` when the token is absent, expired, session-bound to a
    /// different session, or already reserved.
    pub async fn reserve(
        &self,
        session_id: tower_sessions::session::Id,
        token: &str,
    ) -> Result<(), ReserveError> {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::remove_expired(&mut state, now);
        let record = state.records.get_mut(token).ok_or(ReserveError::Conflict)?;
        if record.session_id != session_id.0 || record.reserved || record.expiry <= now {
            return Err(ReserveError::Conflict);
        }
        record.reserved = true;
        Ok(())
    }

    /// Validates, dispatches, and reserves a token as one boundary.
    ///
    /// The callback is invoked while the store lock is held. It must only mark
    /// the request's dispatch boundary and must not await or perform external
    /// work. A rejected callback leaves the token available for retry.
    pub(crate) async fn reserve_and_dispatch(
        &self,
        session_id: tower_sessions::session::Id,
        token: &str,
        dispatch: impl FnOnce() -> Result<(), ()>,
    ) -> Result<(), ReserveError> {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::remove_expired(&mut state, now);
        let record = state.records.get_mut(token).ok_or(ReserveError::Conflict)?;
        if record.session_id != session_id.0 || record.reserved || record.expiry <= now {
            return Err(ReserveError::Conflict);
        }
        dispatch().map_err(|()| ReserveError::Deadline)?;
        record.reserved = true;
        Ok(())
    }

    /// Returns the number of stored anonymous tokens, for invariant-owning tests.
    #[cfg(test)]
    async fn len(&self) -> usize {
        self.state.lock().await.records.len()
    }
}

/// Anonymous token issuance failed because the bounded pool is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueError {
    /// No new token can be admitted without evicting a live token.
    Capacity,
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
        formatter.write_str(CAPACITY_ERROR)
    }
}

impl fmt::Display for ReserveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Conflict => CONFLICT_ERROR,
            Self::Deadline => "login request timed out",
        })
    }
}

impl AnonymousSubmissionTokenStore {
    /// Removes expired tokens using the indexed expiry buckets.
    pub async fn delete_expired(&self) {
        let mut state = self.state.lock().await;
        let now = self.current_time();
        Self::remove_expired(&mut state, now);
    }
}

/// Cleanup contract used by the root supervisor.
#[async_trait]
pub trait SubmissionTokenCleanup: Send + Sync {
    /// Removes expired token records.
    async fn cleanup_expired(&self) -> Result<(), ()>;
}

#[async_trait]
impl SubmissionTokenCleanup for AnonymousSubmissionTokenStore {
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
        ANONYMOUS_CAPACITY, AnonymousSubmissionTokenStore, IssueError, ReserveError, TOKEN_LIFETIME,
    };

    fn store() -> (Arc<Mutex<OffsetDateTime>>, AnonymousSubmissionTokenStore) {
        let now = Arc::new(Mutex::new(OffsetDateTime::UNIX_EPOCH));
        let source = now.clone();
        let store = AnonymousSubmissionTokenStore::with_clock(move || *source.lock().unwrap());
        (now, store)
    }

    fn id(value: i128) -> Id {
        Id(value)
    }

    #[tokio::test]
    async fn issues_one_token_per_session_and_refreshes_expiry() {
        let (now, store) = store();
        let first = store.issue(id(1)).await.expect("first token");
        *now.lock().unwrap() += time::Duration::minutes(5);
        let second = store.issue(id(1)).await.expect("refreshed token");
        assert_eq!(first, second);
        assert_eq!(store.len().await, 1);
        *now.lock().unwrap() += time::Duration::minutes(11);
        store.delete_expired().await;
        assert_eq!(store.len().await, 0);
    }

    #[tokio::test]
    async fn capacity_is_exact_and_does_not_evict_existing_tokens() {
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
            Err(IssueError::Capacity)
        );
        assert_eq!(store.issue(id(0)).await.expect("existing token"), first);
        assert_eq!(store.len().await, ANONYMOUS_CAPACITY);
    }

    #[tokio::test]
    async fn reservation_is_session_bound_and_terminal() {
        let (_, store) = store();
        let token = store.issue(id(1)).await.expect("token");
        assert_eq!(
            store.reserve(id(2), &token).await,
            Err(ReserveError::Conflict)
        );
        store.reserve(id(1), &token).await.expect("reserve token");
        assert_eq!(
            store.reserve(id(1), &token).await,
            Err(ReserveError::Conflict)
        );
    }

    #[tokio::test]
    async fn dispatch_rejection_does_not_consume_the_token() {
        let (_, store) = store();
        let token = store.issue(id(1)).await.expect("token");
        assert_eq!(
            store.reserve_and_dispatch(id(1), &token, || Err(())).await,
            Err(ReserveError::Deadline)
        );
        store
            .reserve_and_dispatch(id(1), &token, || Ok(()))
            .await
            .expect("dispatch token");
        assert_eq!(
            store.reserve(id(1), &token).await,
            Err(ReserveError::Conflict)
        );
    }

    #[tokio::test]
    async fn reservation_reads_the_clock_after_lock_acquisition() {
        let now = Arc::new(std::sync::Mutex::new(OffsetDateTime::UNIX_EPOCH));
        let clock = now.clone();
        let store = AnonymousSubmissionTokenStore::with_clock(move || *clock.lock().unwrap());
        let token = store.issue(id(1)).await.expect("token");
        *now.lock().unwrap() += TOKEN_LIFETIME + time::Duration::seconds(1);
        assert_eq!(
            store.reserve_and_dispatch(id(1), &token, || Ok(())).await,
            Err(ReserveError::Conflict)
        );
    }

    #[tokio::test]
    async fn expired_tokens_are_physically_removed_by_indexed_cleanup() {
        let (now, store) = store();
        store.issue(id(1)).await.expect("token");
        store.issue(id(2)).await.expect("token");
        *now.lock().unwrap() += Duration::from_secs(601);
        store.delete_expired().await;
        assert_eq!(store.len().await, 0);
        assert_eq!(store.issue(id(3)).await.expect("reused capacity").len(), 36);
    }
}
