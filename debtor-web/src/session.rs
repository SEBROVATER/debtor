//! Fallible session operations for authentication and CSRF state.

use tower_sessions::Session;
use uuid::Uuid;

const AUTHENTICATED: &str = "authenticated";
const CSRF: &str = "csrf";

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
