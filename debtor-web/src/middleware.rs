//! Structural HTTP middleware for authentication and response policy.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header::HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::time::Duration;
use tower::BoxError;
use tower_sessions::Session;

use crate::session;

/// Rejects unauthenticated requests before protected handlers are selected.
pub async fn require_authenticated(
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Response {
    match session::authenticated(&session).await {
        Ok(true) => {
            session.set_expiry(Some(session::authenticated_expiry()));
            next.run(request).await
        }
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

/// Maps load-shed failures to a stable retryable response.
pub async fn overload_error(_: BoxError) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "Service temporarily unavailable.",
    )
        .into_response()
}

/// Times out only safe dynamic reads; mutation requests are definitive.
pub async fn safe_read_timeout(request: Request<Body>, next: Next) -> Response {
    safe_read_timeout_with_limits(
        request,
        next,
        Duration::from_secs(30),
        Duration::from_secs(90),
    )
    .await
}

async fn safe_read_timeout_with_limits(
    request: Request<Body>,
    next: Next,
    read_timeout: Duration,
    debt_timeout: Duration,
) -> Response {
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return next.run(request).await;
    }
    let timeout = if request.uri().path().ends_with("/debts") {
        debt_timeout
    } else {
        read_timeout
    };
    match tokio::time::timeout(timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => timeout_response(),
    }
}

/// Applies the fixed login request deadline.
pub async fn login_timeout(request: Request<Body>, next: Next) -> Response {
    match tokio::time::timeout(Duration::from_secs(30), next.run(request)).await {
        Ok(response) => response,
        Err(_) => timeout_response(),
    }
}

/// Keeps probes responsive under a separate two-second deadline.
pub async fn probe_timeout(request: Request<Body>, next: Next) -> Response {
    match tokio::time::timeout(Duration::from_secs(2), next.run(request)).await {
        Ok(response) => response,
        Err(_) => timeout_response(),
    }
}

fn timeout_response() -> Response {
    (StatusCode::GATEWAY_TIMEOUT, "Request timed out.").into_response()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::time::Duration;

    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        routing::any,
    };
    use tower::ServiceExt;

    use super::safe_read_timeout_with_limits;

    #[tokio::test]
    async fn safe_reads_timeout_but_mutations_return_definitive_results() {
        let app = Router::new()
            .route(
                "/{*path}",
                any(|| async {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                    "completed"
                }),
            )
            .layer(middleware::from_fn(|request, next| {
                safe_read_timeout_with_limits(
                    request,
                    next,
                    Duration::from_millis(5),
                    Duration::from_millis(50),
                )
            }));

        let read = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/groups")
                    .body(Body::empty())
                    .expect("read request"),
            )
            .await
            .expect("read response");
        assert_eq!(read.status(), StatusCode::GATEWAY_TIMEOUT);

        let mutation = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/groups")
                    .body(Body::empty())
                    .expect("mutation request"),
            )
            .await
            .expect("mutation response");
        assert_eq!(mutation.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn debt_reads_use_the_longer_budget() {
        let app = Router::new()
            .route(
                "/{*path}",
                any(|| async {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                    "completed"
                }),
            )
            .layer(middleware::from_fn(|request, next| {
                safe_read_timeout_with_limits(
                    request,
                    next,
                    Duration::from_millis(5),
                    Duration::from_millis(50),
                )
            }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/groups/1/debts")
                    .body(Body::empty())
                    .expect("debt request"),
            )
            .await
            .expect("debt response");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
