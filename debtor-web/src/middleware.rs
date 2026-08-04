//! Structural HTTP middleware for authentication and response policy.

use axum::{
    body::Body,
    http::{Request, StatusCode, header::HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::session;

/// Rejects unauthenticated requests before protected handlers are selected.
pub async fn require_authenticated(
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Response {
    match session::authenticated(&session).await {
        Ok(true) => next.run(request).await,
        Ok(false) => Redirect::to("/login").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Session error.").into_response(),
    }
}

/// Adds the common security policy to login and authenticated HTML responses.
pub async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'none'; script-src 'none'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    response
}
