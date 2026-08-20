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
const PARTICIPANT_RESTORE_NOTICES: &str = "participant_restore_notices";
const PARTICIPANT_RESTORE_NOTICE_CAPACITY: usize = 32;
const SPENDING_PREVIEW_GROUP: &str = "spending_preview_group";
const SPENDING_PREVIEW_ID: &str = "spending_preview_id";
const SPENDING_PREVIEW_FIELDS: &str = "spending_preview_fields";
const SPENDING_DELETE_GROUP: &str = "spending_delete_group";
const SPENDING_DELETE_ID: &str = "spending_delete_id";
const SPENDING_DELETE_CURSOR: &str = "spending_delete_cursor";
const SPENDING_DELETE_NEXT: &str = "spending_delete_next";
const SPENDING_DELETE_PREVIOUS: &str = "spending_delete_previous";
const SPENDING_DELETE_CONTROL: &str = "spending_delete_control";
const SPENDING_DELETE_TOKEN: &str = "spending_delete_token";

pub(crate) struct SpendingDeleteBinding {
    pub(crate) group_id: i64,
    pub(crate) spending_id: i64,
    pub(crate) cursor: Option<String>,
    pub(crate) next_focus: Option<i64>,
    pub(crate) previous_focus: Option<i64>,
    pub(crate) control_id: String,
    pub(crate) submission_token: String,
}

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

/// Binds a Spending delete confirmation to one canonical Transactions context.
pub(crate) async fn set_spending_delete_confirmation(
    session: &Session,
    binding: SpendingDeleteBinding,
) -> Result<(), SessionError> {
    session
        .insert(SPENDING_DELETE_GROUP, binding.group_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_ID, binding.spending_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_CURSOR, binding.cursor)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_NEXT, binding.next_focus)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_PREVIOUS, binding.previous_focus)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_CONTROL, binding.control_id)
        .await
        .map_err(|_| SessionError)?;
    session
        .insert(SPENDING_DELETE_TOKEN, binding.submission_token)
        .await
        .map_err(|_| SessionError)
}

/// Reads the session-bound Spending delete confirmation context.
pub(crate) async fn spending_delete_confirmation(
    session: &Session,
) -> Result<
    Option<(
        i64,
        i64,
        Option<String>,
        Option<i64>,
        Option<i64>,
        String,
        String,
    )>,
    SessionError,
> {
    let group_id = session
        .get::<i64>(SPENDING_DELETE_GROUP)
        .await
        .map_err(|_| SessionError)?;
    let spending_id = session
        .get::<i64>(SPENDING_DELETE_ID)
        .await
        .map_err(|_| SessionError)?;
    let cursor = session
        .get::<Option<String>>(SPENDING_DELETE_CURSOR)
        .await
        .map_err(|_| SessionError)?
        .flatten();
    let next_focus = session
        .get::<Option<i64>>(SPENDING_DELETE_NEXT)
        .await
        .map_err(|_| SessionError)?
        .flatten();
    let previous_focus = session
        .get::<Option<i64>>(SPENDING_DELETE_PREVIOUS)
        .await
        .map_err(|_| SessionError)?
        .flatten();
    let control_id = session
        .get::<String>(SPENDING_DELETE_CONTROL)
        .await
        .map_err(|_| SessionError)?;
    let submission_token = session
        .get::<String>(SPENDING_DELETE_TOKEN)
        .await
        .map_err(|_| SessionError)?;
    Ok(group_id
        .zip(spending_id)
        .zip(control_id)
        .zip(submission_token)
        .map(
            |(((group_id, spending_id), control_id), submission_token)| {
                (
                    group_id,
                    spending_id,
                    cursor,
                    next_focus,
                    previous_focus,
                    control_id,
                    submission_token,
                )
            },
        ))
}

/// Clears the server-owned Spending delete confirmation state.
pub(crate) async fn clear_spending_delete_confirmation(
    session: &Session,
) -> Result<(), SessionError> {
    session
        .remove::<i64>(SPENDING_DELETE_GROUP)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<i64>(SPENDING_DELETE_ID)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Option<String>>(SPENDING_DELETE_CURSOR)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Option<i64>>(SPENDING_DELETE_NEXT)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<Option<i64>>(SPENDING_DELETE_PREVIOUS)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<String>(SPENDING_DELETE_CONTROL)
        .await
        .map_err(|_| SessionError)?;
    session
        .remove::<String>(SPENDING_DELETE_TOKEN)
        .await
        .map_err(|_| SessionError)?;
    Ok(())
}

/// Returns the server-owned Delete control identity for a pending confirmation.
pub(crate) async fn spending_delete_focus(session: &Session) -> Result<Option<i64>, SessionError> {
    let Some(control_id) = session
        .get::<String>(SPENDING_DELETE_CONTROL)
        .await
        .map_err(|_| SessionError)?
    else {
        return Ok(None);
    };
    let Some(id) = control_id
        .strip_prefix("spending-")
        .and_then(|value| value.strip_suffix("-delete"))
        .and_then(|value| value.parse::<i64>().ok())
    else {
        return Ok(None);
    };
    Ok((id > 0).then_some(id))
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

/// Binds the restored Participant row focus to the authenticated session.
pub(crate) async fn set_participant_restore_focus(
    session: &Session,
    group_id: i64,
    participant_id: i64,
) -> Result<String, SessionError> {
    set_participant_restore_notice(session, group_id, participant_id, true).await
}

/// Binds a failed Participant restore focus target to the authenticated session.
pub(crate) async fn set_participant_restore_failure_focus(
    session: &Session,
    group_id: i64,
    participant_id: i64,
) -> Result<String, SessionError> {
    set_participant_restore_notice(session, group_id, participant_id, false).await
}

async fn set_participant_restore_notice(
    session: &Session,
    group_id: i64,
    participant_id: i64,
    succeeded: bool,
) -> Result<String, SessionError> {
    let mut notices = session
        .get::<Vec<(String, i64, i64, bool)>>(PARTICIPANT_RESTORE_NOTICES)
        .await
        .map_err(|_| SessionError)?
        .unwrap_or_default();
    if notices.len() == PARTICIPANT_RESTORE_NOTICE_CAPACITY {
        notices.remove(0);
    }
    let nonce = Uuid::new_v4().to_string();
    notices.push((nonce.clone(), group_id, participant_id, succeeded));
    session
        .insert(PARTICIPANT_RESTORE_NOTICES, notices)
        .await
        .map_err(|_| SessionError)?;
    Ok(nonce)
}

/// Consumes one server-owned Participant restore notice for its Group and redirect nonce.
pub(crate) async fn take_participant_restore_notice(
    session: &Session,
    group_id: i64,
    nonce: &str,
) -> Result<Option<(i64, bool)>, SessionError> {
    let mut notices = session
        .get::<Vec<(String, i64, i64, bool)>>(PARTICIPANT_RESTORE_NOTICES)
        .await
        .map_err(|_| SessionError)?
        .unwrap_or_default();
    let Some(index) = notices
        .iter()
        .position(|(stored_nonce, stored_group, _, _)| {
            stored_nonce == nonce && *stored_group == group_id
        })
    else {
        return Ok(None);
    };
    let (_, _, participant_id, succeeded) = notices.remove(index);
    session
        .insert(PARTICIPANT_RESTORE_NOTICES, notices)
        .await
        .map_err(|_| SessionError)?;
    Ok(Some((participant_id, succeeded)))
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
        establish, flush, set_participant_restore_failure_focus, set_participant_restore_focus,
        set_spending_preview, spending_preview_matches, take_matching_spending_preview,
        take_participant_restore_notice,
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

    #[tokio::test]
    async fn participant_restore_focus_is_server_owned_and_single_use() {
        let store = Arc::new(ReapingMemoryStore::default());
        let session = Session::new(None, store, Some(authenticated_expiry()));
        session.save().await.expect("save authenticated session");

        let success = set_participant_restore_focus(&session, 1, 7)
            .await
            .expect("set restore focus");
        assert_eq!(
            take_participant_restore_notice(&session, 1, &success)
                .await
                .expect("take restore focus"),
            Some((7, true))
        );
        let success = set_participant_restore_focus(&session, 1, 7)
            .await
            .expect("reset restore focus");
        assert_eq!(
            take_participant_restore_notice(&session, 2, &success)
                .await
                .expect("reject wrong group restore focus"),
            None
        );
        assert_eq!(
            take_participant_restore_notice(&session, 1, &success)
                .await
                .expect("consume matching restore focus"),
            Some((7, true))
        );

        let failure = set_participant_restore_failure_focus(&session, 1, 7)
            .await
            .expect("set restore failure focus");
        assert_eq!(
            take_participant_restore_notice(&session, 2, &failure)
                .await
                .expect("reject wrong group failure focus"),
            None
        );
        assert_eq!(
            take_participant_restore_notice(&session, 1, &failure)
                .await
                .expect("consume matching restore failure focus"),
            Some((7, false))
        );
        assert_eq!(
            take_participant_restore_notice(&session, 1, &failure)
                .await
                .expect("failure focus is single use"),
            None
        );

        let first = set_participant_restore_focus(&session, 1, 7)
            .await
            .expect("set first concurrent restore focus");
        let second = set_participant_restore_focus(&session, 1, 8)
            .await
            .expect("set second concurrent restore focus");
        assert_eq!(
            take_participant_restore_notice(&session, 1, &first)
                .await
                .expect("consume first concurrent restore focus"),
            Some((7, true))
        );
        assert_eq!(
            take_participant_restore_notice(&session, 1, &second)
                .await
                .expect("consume second concurrent restore focus"),
            Some((8, true))
        );
    }
}
