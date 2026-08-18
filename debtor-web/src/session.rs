//! Fallible session operations for authentication and CSRF state.

use std::sync::OnceLock;
use time::Duration;
use tower_sessions::{Expiry, Session};
use uuid::Uuid;

pub(crate) const AUTHENTICATED: &str = "authenticated";
const CSRF: &str = "csrf";
const GROUP_DELETE_ID: &str = "group_delete_id";
const GROUP_DELETE_PARTICIPANTS: &str = "group_delete_participants";
const GROUP_DELETE_TOKEN: &str = "group_delete_token";
const GROUP_RESTORE_FOCUS: &str = "group_restore_focus";
const SPENDING_PREVIEW_GROUP: &str = "spending_preview_group";
const SPENDING_PREVIEW_ID: &str = "spending_preview_id";
const SPENDING_PREVIEW_FIELDS: &str = "spending_preview_fields";

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

static SPENDING_APPROVAL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

pub(crate) fn spending_approval_lock() -> &'static tokio::sync::Mutex<()> {
    SPENDING_APPROVAL_LOCK.get_or_init(tokio::sync::Mutex::default)
}

pub(crate) async fn set_spending_preview(
    session: &Session,
    group_id: i64,
    spending_id: Option<i64>,
    fields: Vec<(String, String)>,
) -> Result<(), SessionError> {
    session
        .insert(SPENDING_PREVIEW_GROUP, group_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_PREVIEW_ID, spending_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_PREVIEW_FIELDS, fields)
        .await
        .map_err(|_| SessionError)
}

pub(crate) async fn take_matching_spending_preview(
    session: &Session,
    group_id: i64,
    spending_id: Option<i64>,
    fields: &[(String, String)],
) -> Result<bool, SessionError> {
    let matches = spending_preview_matches(session, group_id, spending_id, fields).await?;
    if matches {
        clear_spending_preview(session).await?;
    }
    Ok(matches)
}

pub(crate) async fn spending_preview_matches(
    session: &Session,
    group_id: i64,
    spending_id: Option<i64>,
    fields: &[(String, String)],
) -> Result<bool, SessionError> {
    Ok(session
        .get::<i64>(SPENDING_PREVIEW_GROUP)
        .await
        .map_err(|_| SessionError)?
        .is_some_and(|stored_group| stored_group == group_id)
        && session
            .get::<Option<i64>>(SPENDING_PREVIEW_ID)
            .await
            .map_err(|_| SessionError)?
            .flatten()
            == spending_id
        && session
            .get::<Vec<(String, String)>>(SPENDING_PREVIEW_FIELDS)
            .await
            .map_err(|_| SessionError)?
            .is_some_and(|stored_fields| stored_fields == fields))
}

pub(crate) async fn clear_spending_preview(session: &Session) -> Result<(), SessionError> {
    session
        .remove::<i64>(SPENDING_PREVIEW_GROUP)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Option<i64>>(SPENDING_PREVIEW_ID)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Vec<(String, String)>>(SPENDING_PREVIEW_FIELDS)
        .await
        .map(|_| ())
        .map_err(|_| SessionError)
}

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

/// Binds the disclosed Group deletion snapshot to the authenticated session.
pub(crate) async fn set_group_delete_confirmation(
    session: &Session,
    group_id: i64,
    participant_ids: Vec<i64>,
    submission_token: &str,
) -> Result<(), SessionError> {
    session
        .insert(GROUP_DELETE_ID, group_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(GROUP_DELETE_PARTICIPANTS, participant_ids)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(GROUP_DELETE_TOKEN, submission_token.to_owned())
        .await
        .map_err(|_| SessionError)
}

/// Reads the session-bound Group deletion snapshot.
pub(crate) async fn group_delete_confirmation(
    session: &Session,
) -> Result<Option<(i64, Vec<i64>, String)>, SessionError> {
    let group_id = session
        .get::<i64>(GROUP_DELETE_ID)
        .await
        .map_err(|_| SessionError)?;
    let participant_ids = session
        .get::<Vec<i64>>(GROUP_DELETE_PARTICIPANTS)
        .await
        .map_err(|_| SessionError)?;
    let submission_token = session
        .get::<String>(GROUP_DELETE_TOKEN)
        .await
        .map_err(|_| SessionError)?;
    Ok(group_id
        .zip(participant_ids)
        .zip(submission_token)
        .map(|((id, participant_ids), token)| (id, participant_ids, token)))
}

/// Removes the server-owned Group deletion confirmation state.
pub(crate) async fn clear_group_delete_confirmation(session: &Session) -> Result<(), SessionError> {
    session
        .remove::<i64>(GROUP_DELETE_ID)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Vec<i64>>(GROUP_DELETE_PARTICIPANTS)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<String>(GROUP_DELETE_TOKEN)
        .await
        .map_err(|_| SessionError)?;
    Ok(())
}

/// Binds the restored Group row focus to the authenticated session.
pub(crate) async fn set_restore_focus(
    session: &Session,
    group_id: i64,
) -> Result<(), SessionError> {
    session
        .insert(GROUP_RESTORE_FOCUS, group_id)
        .await
        .map_err(|_| SessionError)
}

/// Consumes the server-owned restored Group focus target.
pub(crate) async fn take_restore_focus(session: &Session) -> Result<Option<i64>, SessionError> {
    let focus = session
        .get::<i64>(GROUP_RESTORE_FOCUS)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<i64>(GROUP_RESTORE_FOCUS)
        .await
        .map_err(|_| SessionError)?;
    Ok(focus)
}

/// Rotates and durably establishes an authenticated session.
pub(crate) async fn establish(session: &Session) -> Result<(), SessionError> {
    // Cycle before saving so the authenticated record can never reuse the
    // anonymous session identifier.
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
    // Delete before clearing local state so a failed store operation cannot
    // make the response look like a successful browser logout.
    session.delete().await.map_err(|_| SessionError)?;
    session.flush().await.map_err(|_| SessionError)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use tower_sessions::{Session, SessionStore};

    use super::{
        AUTHENTICATED, anonymous_expiry, authenticated, authenticated_expiry, csrf_token,
        establish, flush, set_spending_preview, spending_preview_matches,
        take_matching_spending_preview,
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

    #[tokio::test]
    async fn spending_preview_binding_includes_existing_spending_identity() {
        let store = Arc::new(ReapingMemoryStore::default());
        let session = Session::new(None, store, Some(authenticated_expiry()));
        session.save().await.expect("save authenticated session");
        let fields = vec![("total".to_owned(), "10.00".to_owned())];

        set_spending_preview(&session, 7, Some(11), fields.clone())
            .await
            .expect("set edit preview");
        assert!(
            take_matching_spending_preview(&session, 7, Some(11), &fields)
                .await
                .expect("take matching edit preview")
        );

        set_spending_preview(&session, 7, Some(11), fields.clone())
            .await
            .expect("reset edit preview");
        assert!(
            !take_matching_spending_preview(&session, 7, Some(12), &fields)
                .await
                .expect("reject wrong Spending preview")
        );
        assert!(
            spending_preview_matches(&session, 7, Some(11), &fields)
                .await
                .expect("retain edit preview after mismatch")
        );
    }
}
