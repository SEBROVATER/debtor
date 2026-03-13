use crate::web::router::RouteSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthOutcome {
    Allowed,
    RedirectToLogin,
}

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub session_id: Option<String>,
}

impl SessionContext {
    pub fn is_authenticated(&self) -> bool {
        self.session_id.is_some()
    }
}

pub fn enforce_auth(route: &RouteSpec, is_authenticated: bool) -> AuthOutcome {
    if route.requires_auth && !is_authenticated {
        AuthOutcome::RedirectToLogin
    } else {
        AuthOutcome::Allowed
    }
}

pub fn extract_session_cookie(cookie_header: Option<&str>, cookie_name: &str) -> Option<String> {
    let header = cookie_header?;
    header
        .split(';')
        .map(|pair| pair.trim())
        .find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let name = parts.next()?.trim();
            let value = parts.next()?.trim();
            if name == cookie_name && !value.is_empty() {
                Some(value.to_string())
            } else {
                None
            }
        })
}
