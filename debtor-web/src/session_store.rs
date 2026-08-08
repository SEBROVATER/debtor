//! Bounded in-memory session storage with deterministic expiry handling.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    sync::Arc,
};

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tower_sessions::{
    ExpiredDeletion, SessionStore,
    session::{Id, Record},
    session_store,
};

use crate::session::AUTHENTICATED;

pub(crate) const ANONYMOUS_CAPACITY: usize = 4_096;
pub(crate) const AUTHENTICATED_CAPACITY: usize = 32;

const CAPACITY_ERROR: &str = "anonymous session capacity reached";

#[derive(Debug)]
struct StoreState {
    records: HashMap<Id, Record>,
    anonymous_count: usize,
    authenticated_count: usize,
    expiry_index: BTreeMap<OffsetDateTime, BTreeSet<i128>>,
}

/// A process-local session store with bounded anonymous admission.
pub struct ReapingMemoryStore {
    state: Arc<Mutex<StoreState>>,
    now: Arc<dyn Fn() -> OffsetDateTime + Send + Sync>,
}

impl Clone for ReapingMemoryStore {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            now: self.now.clone(),
        }
    }
}

impl fmt::Debug for ReapingMemoryStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReapingMemoryStore")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for ReapingMemoryStore {
    fn default() -> Self {
        Self::with_clock(OffsetDateTime::now_utc)
    }
}

impl ReapingMemoryStore {
    pub(crate) fn with_clock<F>(clock: F) -> Self
    where
        F: Fn() -> OffsetDateTime + Send + Sync + 'static,
    {
        Self {
            state: Arc::new(Mutex::new(StoreState {
                records: HashMap::new(),
                anonymous_count: 0,
                authenticated_count: 0,
                expiry_index: BTreeMap::new(),
            })),
            now: Arc::new(clock),
        }
    }

    fn current_time(&self) -> OffsetDateTime {
        (self.now)()
    }

    fn is_anonymous(record: &Record) -> bool {
        record
            .data
            .get(AUTHENTICATED)
            .is_none_or(|value| value.as_bool() != Some(true))
    }

    fn increment_count(state: &mut StoreState, record: &Record) {
        if Self::is_anonymous(record) {
            state.anonymous_count += 1;
        } else {
            state.authenticated_count += 1;
        }
    }

    fn decrement_count(state: &mut StoreState, record: &Record) {
        if Self::is_anonymous(record) {
            state.anonymous_count -= 1;
        } else {
            state.authenticated_count -= 1;
        }
    }

    fn index_record(state: &mut StoreState, record: &Record) {
        state
            .expiry_index
            .entry(record.expiry_date)
            .or_default()
            .insert(record.id.0);
    }

    fn unindex_record(state: &mut StoreState, record: &Record) {
        let remove_bucket = state
            .expiry_index
            .get_mut(&record.expiry_date)
            .is_some_and(|ids| {
                ids.remove(&record.id.0);
                ids.is_empty()
            });
        if remove_bucket {
            state.expiry_index.remove(&record.expiry_date);
        }
    }

    fn remove_record(state: &mut StoreState, id: &Id) -> Option<Record> {
        let record = state.records.remove(id)?;
        Self::unindex_record(state, &record);
        Self::decrement_count(state, &record);
        Some(record)
    }

    fn remove_expired(state: &mut StoreState, now: OffsetDateTime) {
        while let Some((&expiry, _)) = state.expiry_index.first_key_value() {
            if expiry > now {
                break;
            }
            let ids = state.expiry_index.remove(&expiry).unwrap_or_default();
            for id in ids {
                let id = Id(id);
                if let Some(record) = state.records.remove(&id) {
                    Self::decrement_count(state, &record);
                }
            }
        }
    }

    fn capacity_error() -> session_store::Error {
        session_store::Error::Backend(CAPACITY_ERROR.to_owned())
    }

    #[cfg(test)]
    pub(crate) async fn counts(&self) -> (usize, usize, usize) {
        let state = self.state.lock().await;
        (
            state.records.len(),
            state.anonymous_count,
            state.authenticated_count,
        )
    }
}

#[async_trait]
impl SessionStore for ReapingMemoryStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let now = self.current_time();
        let mut state = self.state.lock().await;
        Self::remove_expired(&mut state, now);

        let anonymous = Self::is_anonymous(record);
        if anonymous && state.anonymous_count >= ANONYMOUS_CAPACITY {
            return Err(Self::capacity_error());
        }
        if !anonymous && state.authenticated_count >= AUTHENTICATED_CAPACITY {
            return Err(Self::capacity_error());
        }

        while state.records.contains_key(&record.id) {
            record.id = Id::default();
        }

        state.records.insert(record.id, record.clone());
        Self::increment_count(&mut state, record);
        Self::index_record(&mut state, record);
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let now = self.current_time();
        let mut state = self.state.lock().await;
        Self::remove_expired(&mut state, now);

        let new_anonymous = Self::is_anonymous(record);
        let old = state.records.get(&record.id).cloned();
        let old_anonymous = old.as_ref().is_some_and(Self::is_anonymous);
        let old_authenticated = old.as_ref().is_some_and(|value| !Self::is_anonymous(value));
        let available_anonymous = state.anonymous_count - usize::from(old_anonymous);
        let available_authenticated = state.authenticated_count - usize::from(old_authenticated);
        if new_anonymous && available_anonymous >= ANONYMOUS_CAPACITY {
            return Err(Self::capacity_error());
        }
        if !new_anonymous && available_authenticated >= AUTHENTICATED_CAPACITY {
            return Err(Self::capacity_error());
        }
        if let Some(old) = old.as_ref() {
            Self::unindex_record(&mut state, old);
            Self::decrement_count(&mut state, old);
        }
        state.records.insert(record.id, record.clone());
        Self::increment_count(&mut state, record);
        Self::index_record(&mut state, record);
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let now = self.current_time();
        let mut state = self.state.lock().await;
        let Some(record) = state.records.get(session_id).cloned() else {
            return Ok(None);
        };
        if record.expiry_date <= now {
            Self::remove_record(&mut state, session_id);
            return Ok(None);
        }
        Ok(Some(record))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let mut state = self.state.lock().await;
        Self::remove_record(&mut state, session_id);
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for ReapingMemoryStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let now = self.current_time();
        let mut state = self.state.lock().await;
        Self::remove_expired(&mut state, now);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use serde_json::json;
    use time::{Duration, OffsetDateTime};
    use tower_sessions::{
        SessionStore,
        session::{Id, Record},
        session_store::ExpiredDeletion,
    };

    use super::{ANONYMOUS_CAPACITY, AUTHENTICATED_CAPACITY, ReapingMemoryStore};

    fn clock() -> (Arc<Mutex<OffsetDateTime>>, ReapingMemoryStore) {
        let now = Arc::new(Mutex::new(OffsetDateTime::UNIX_EPOCH));
        let source = now.clone();
        let store = ReapingMemoryStore::with_clock(move || *source.lock().unwrap());
        (now, store)
    }

    fn record(id: Id, expiry_date: OffsetDateTime) -> Record {
        Record {
            id,
            data: HashMap::default(),
            expiry_date,
        }
    }

    #[tokio::test]
    async fn create_save_load_delete_and_restart_invalidation_work() {
        let (_, store) = clock();
        let mut record = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        store.create(&mut record).await.unwrap();
        assert_eq!(store.load(&record.id).await.unwrap(), Some(record.clone()));

        record.data.insert("value".into(), json!(42));
        store.save(&record).await.unwrap();
        assert_eq!(store.load(&record.id).await.unwrap(), Some(record.clone()));

        store.delete(&record.id).await.unwrap();
        assert!(store.load(&record.id).await.unwrap().is_none());
        assert!(
            ReapingMemoryStore::default()
                .load(&record.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn create_retries_id_collisions_without_evicting() {
        let (_, store) = clock();
        let mut first = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        store.create(&mut first).await.unwrap();
        let first_id = first.id;
        let mut second = record(first_id, OffsetDateTime::UNIX_EPOCH + Duration::hours(1));
        store.create(&mut second).await.unwrap();

        assert_ne!(first.id, second.id);
        assert_eq!(store.counts().await, (2, 2, 0));
    }

    #[tokio::test]
    async fn anonymous_capacity_is_exact_and_authenticated_records_are_exempt() {
        let (_, store) = clock();
        for _ in 0..ANONYMOUS_CAPACITY {
            let mut record = record(
                Id::default(),
                OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
            );
            store.create(&mut record).await.unwrap();
        }
        let mut rejected = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        assert!(store.create(&mut rejected).await.is_err());

        let mut authenticated = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        authenticated
            .data
            .insert("authenticated".into(), json!(true));
        store.create(&mut authenticated).await.unwrap();
        assert_eq!(
            store.counts().await,
            (ANONYMOUS_CAPACITY + 1, ANONYMOUS_CAPACITY, 1)
        );
    }

    #[tokio::test]
    async fn authenticated_capacity_is_exact_without_eviction() {
        let (_, store) = clock();
        for _ in 0..AUTHENTICATED_CAPACITY {
            let mut record = record(
                Id::default(),
                OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
            );
            record.data.insert("authenticated".into(), json!(true));
            store.create(&mut record).await.unwrap();
        }
        let first_id = {
            let state = store.state.lock().await;
            *state.records.keys().next().unwrap()
        };
        let first = store.load(&first_id).await.unwrap().unwrap();
        let mut rejected = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        rejected.data.insert("authenticated".into(), json!(true));
        assert!(store.create(&mut rejected).await.is_err());
        assert!(store.load(&first.id).await.unwrap().is_some());
        assert_eq!(
            store.counts().await,
            (AUTHENTICATED_CAPACITY, 0, AUTHENTICATED_CAPACITY)
        );
    }

    #[tokio::test]
    async fn expiry_is_lazy_and_recovers_capacity() {
        let (now, store) = clock();
        for _ in 0..ANONYMOUS_CAPACITY {
            let mut record = record(
                Id::default(),
                OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
            );
            store.create(&mut record).await.unwrap();
        }
        *now.lock().unwrap() = OffsetDateTime::UNIX_EPOCH + Duration::hours(1);
        let mut record = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(2),
        );
        store.create(&mut record).await.unwrap();
        assert_eq!(store.counts().await, (1, 1, 0));
    }

    #[tokio::test]
    async fn load_and_explicit_deletion_physically_remove_expired_records() {
        let (now, store) = clock();
        let mut expired = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        store.create(&mut expired).await.unwrap();
        *now.lock().unwrap() = OffsetDateTime::UNIX_EPOCH + Duration::hours(1);
        assert!(store.load(&expired.id).await.unwrap().is_none());
        assert_eq!(store.counts().await, (0, 0, 0));

        let mut next = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(2),
        );
        store.create(&mut next).await.unwrap();
        store.delete_expired().await.unwrap();
        assert_eq!(store.counts().await, (1, 1, 0));
        store.delete(&next.id).await.unwrap();
        assert_eq!(store.counts().await, (0, 0, 0));
    }

    #[tokio::test]
    async fn promotion_and_demotion_update_anonymous_count() {
        let (_, store) = clock();
        let mut record = record(
            Id::default(),
            OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
        );
        store.create(&mut record).await.unwrap();
        assert_eq!(store.counts().await, (1, 1, 0));
        record.data.insert("authenticated".into(), json!(true));
        store.save(&record).await.unwrap();
        assert_eq!(store.counts().await, (1, 0, 1));
        record.data.remove("authenticated");
        store.save(&record).await.unwrap();
        assert_eq!(store.counts().await, (1, 1, 0));
    }
}
