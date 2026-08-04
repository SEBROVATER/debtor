//! Fallible session operations for authentication and CSRF state.

use time::Duration;
use tower_sessions::{Expiry, Session};
use uuid::Uuid;

pub(crate) const AUTHENTICATED: &str = "authenticated";
const CSRF: &str = "csrf";

/// Returns the fixed expiry policy for anonymous sessions.
pub fn anonymous_expiry() -> Expiry {
    Expiry::OnInactivity(Duration::minutes(10))
}

pub(crate) fn authenticated_expiry() -> Expiry {
    Expiry::OnInactivity(Duration::days(30))
}

/// A session store operation failed.
#[derive(Debug)]
pub(crate) struct SessionError;

/// Returns whether this session is authenticated.
pub(crate) async fn authenticated(session: &Session) -> Result<bool, SessionError> {
    Ok(session
        .get::<bool>(AUTHENTICATED)
        .await
        .map_err(|_| SessionError)?
        .unwrap_or(false))
}

/// Returns the session CSRF token, creating one for a new session.
pub(crate) async fn csrf_token(session: &Session) -> Result<String, SessionError> {
    if let Some(value) = session
        .get::<String>(CSRF)
        .await
        .map_err(|_| SessionError)?
    {
        return Ok(value);
    }

    let value = Uuid::new_v4().to_string();
    session
        .insert(CSRF, value.clone())
        .await
        .map_err(|_| SessionError)?;
    Ok(value)
}

/// Checks the supplied token against the session-backed synchronizer token.
pub(crate) async fn matches_csrf(session: &Session, supplied: &str) -> Result<bool, SessionError> {
    Ok(session
        .get::<String>(CSRF)
        .await
        .map_err(|_| SessionError)?
        .is_some_and(|value| value == supplied))
}

/// Rotates and durably establishes an authenticated session.
pub(crate) async fn establish(session: &Session) -> Result<(), SessionError> {
    // Preserve this order so no successful login redirects before its new state is durable.
    session.cycle_id().await.map_err(|_| SessionError)?;
    session.set_expiry(Some(authenticated_expiry()));
    session
        .insert(CSRF, Uuid::new_v4().to_string())
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(AUTHENTICATED, true)
        .await
        .map_err(|_| SessionError)?;
    session.save().await.map_err(|_| SessionError)
}

/// Removes all session state.
pub(crate) async fn flush(session: &Session) -> Result<(), SessionError> {
    session.flush().await.map_err(|_| SessionError)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use tower_sessions::{Session, SessionStore};

    use super::{
        AUTHENTICATED, anonymous_expiry, authenticated, authenticated_expiry, csrf_token,
        establish, flush,
    };
    use crate::session_store::ReapingMemoryStore;

    #[tokio::test]
    async fn establish_rotates_id_and_csrf_and_flush_removes_session() {
        let store = Arc::new(ReapingMemoryStore::default());
        let session = Session::new(None, store.clone(), Some(anonymous_expiry()));
        let old_csrf = csrf_token(&session).await.expect("anonymous CSRF");
        session.save().await.expect("save anonymous session");
        let old_id = session.id().expect("anonymous ID");

        establish(&session).await.expect("establish session");
        let new_id = session.id().expect("authenticated ID");
        assert_ne!(old_id, new_id);
        assert_ne!(old_csrf, csrf_token(&session).await.expect("rotated CSRF"));
        assert!(authenticated(&session).await.expect("auth marker"));
        assert_eq!(session.expiry(), Some(authenticated_expiry()));
        assert!(
            store
                .load(&old_id)
                .await
                .expect("old session lookup")
                .is_none()
        );
        assert!(
            store
                .load(&new_id)
                .await
                .expect("new session lookup")
                .is_some()
        );

        flush(&session).await.expect("flush session");
        assert!(store.load(&new_id).await.expect("flushed lookup").is_none());
        assert!(
            !session
                .get::<bool>(AUTHENTICATED)
                .await
                .expect("flushed auth marker")
                .unwrap_or(false)
        );
    }
}
