use debtor::auth::middleware::{AuthOutcome, enforce_auth};
use debtor::web::router::route_specs;

#[test]
fn unauthenticated_requests_redirect_for_protected_routes() {
    let routes = route_specs();
    assert!(!routes.is_empty(), "expected at least one route spec");

    for route in routes {
        let outcome = enforce_auth(&route, false);
        if route.requires_auth {
            assert_eq!(
                outcome,
                AuthOutcome::RedirectToLogin,
                "{:?} {} should redirect",
                route.method,
                route.path
            );
        } else {
            assert_eq!(
                outcome,
                AuthOutcome::Allowed,
                "{:?} {} should be public",
                route.method,
                route.path
            );
        }
    }
}
