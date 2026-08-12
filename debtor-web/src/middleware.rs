//! Structural HTTP middleware for authentication and response policy.

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Method, Request, StatusCode, header::HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tower::BoxError;
use tower_sessions::Session;

use crate::session;

const MUTATION_DEADLINE: Duration = Duration::from_secs(30);

/// Shared absolute deadline for work before a ledger mutation is dispatched.
#[derive(Clone)]
pub(crate) struct MutationPreflight {
    deadline: tokio::time::Instant,
    dispatched: Arc<AtomicBool>,
    login_route: bool,
}

impl MutationPreflight {
    fn new(login_route: bool) -> Self {
        Self {
            deadline: tokio::time::Instant::now() + MUTATION_DEADLINE,
            dispatched: Arc::new(AtomicBool::new(false)),
            login_route,
        }
    }

    /// Runs one pre-dispatch operation within the request's remaining budget.
    pub(crate) async fn wait<T>(&self, future: impl Future<Output = T>) -> Result<T, Response> {
        if self.dispatched.load(Ordering::Acquire) {
            return Err(self.timeout_response());
        }
        tokio::time::timeout_at(self.deadline, future)
            .await
            .map_err(|_| self.timeout_response())
    }

    /// Irreversibly marks the request as dispatched to its state-changing operation.
    #[allow(clippy::result_large_err)]
    pub(crate) fn dispatch(&self) -> Result<(), Response> {
        if tokio::time::Instant::now() >= self.deadline
            || self.dispatched.swap(true, Ordering::AcqRel)
        {
            return Err(self.timeout_response());
        }
        Ok(())
    }

    fn timeout_response(&self) -> Response {
        if self.login_route {
            crate::handlers::response::login_timeout()
        } else {
            timeout_response()
        }
    }
}

/// Adds one preflight object to protected unsafe requests only.
pub async fn mutation_preflight(mut request: Request<Body>, next: Next) -> Response {
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        let login_route = request.uri().path() == "/login";
        request
            .extensions_mut()
            .insert(MutationPreflight::new(login_route));
    }
    next.run(request).await
}

/// Rejects unauthenticated requests before protected handlers are selected.
pub async fn require_authenticated(
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Response {
    let authenticated = if let Some(preflight) = request.extensions().get::<MutationPreflight>() {
        match preflight.wait(session::authenticated(&session)).await {
            Ok(value) => value,
            Err(response) => return response,
        }
    } else {
        session::authenticated(&session).await
    };
    match authenticated {
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
            "default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    response
}

/// Records only safe HTTP response metadata.
pub async fn http_observability(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let route = matched_route(&request);
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status();
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    record_http_response(&method, &route, status, latency_ms);
    response
}

fn matched_route(request: &Request<Body>) -> String {
    request
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", MatchedPath::as_str)
        .to_owned()
}

fn record_http_response(method: &Method, route: &str, status: StatusCode, latency_ms: u64) {
    if status.is_server_error() {
        tracing::warn!(
            target: "debtor.http",
            event = "http_response",
            method = %method,
            route = %route,
            status = status.as_u16(),
            latency_ms,
        );
    } else {
        tracing::info!(
            target: "debtor.http",
            event = "http_response",
            method = %method,
            route = %route,
            status = status.as_u16(),
            latency_ms,
        );
    }
}

/// Maps load-shed failures to a stable retryable response.
pub async fn overload_error(_: BoxError) -> Response {
    tracing::warn!(
        target: "debtor.http",
        event = "request_admission_rejected",
        category = "concurrency",
        count = 1_u64,
    );
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
    if request.method() == Method::POST {
        return next.run(request).await;
    }
    match tokio::time::timeout(Duration::from_secs(30), next.run(request)).await {
        Ok(response) => response,
        Err(_) => login_timeout_response(),
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

fn login_timeout_response() -> Response {
    crate::handlers::response::login_error_response(
        StatusCode::GATEWAY_TIMEOUT,
        "Sign-in request timed out. Try again.",
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::{
        io::Write,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        routing::any,
    };
    use tower::ServiceExt;
    use tracing_subscriber::fmt::MakeWriter;

    use super::{matched_route, record_http_response, safe_read_timeout_with_limits};

    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    struct Writer(Arc<Mutex<Vec<u8>>>);

    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("writer lock").extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for SharedWriter {
        type Writer = Writer;

        fn make_writer(&'a self) -> Self::Writer {
            Writer(self.0.clone())
        }
    }

    #[test]
    fn http_event_contains_safe_fields_without_query_or_body_data() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(SharedWriter(buffer.clone()))
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            record_http_response(&Method::GET, "/groups/{id}", StatusCode::OK, 12);
        });
        let output =
            String::from_utf8(buffer.lock().expect("log buffer").clone()).expect("UTF-8 logs");
        assert!(output.contains("http_response"));
        assert!(output.contains("method=GET"));
        assert!(output.contains("route=/groups/{id}"));
        assert!(output.contains("status=200"));
        assert!(output.contains("latency_ms=12"));
        assert!(!output.contains("password=sentinel"));
        assert!(!output.contains("csrf=sentinel"));
    }

    #[test]
    fn route_fallback_never_uses_the_raw_uri_or_query() {
        let request = Request::builder()
            .uri("/login?password=sentinel")
            .body(Body::empty())
            .expect("request");
        assert_eq!(matched_route(&request), "unmatched");
    }

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
