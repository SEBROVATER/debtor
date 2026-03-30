use debtor::web::router::{RouteMethod, route_specs};

#[test]
fn protected_routes_require_auth_and_state_changes_have_csrf() {
    let routes = route_specs();
    assert!(!routes.is_empty(), "expected routes");

    for route in routes {
        if route.path == "/login" || route.path == "/health" {
            assert!(!route.requires_auth, "{} should be public", route.path);
        } else {
            assert!(route.requires_auth, "{} should require auth", route.path);
        }

        if route.method.is_state_changing() {
            assert!(
                route.csrf_protected,
                "{:?} {} should be csrf protected",
                route.method, route.path
            );
        }

        if matches!(route.method, RouteMethod::Get) {
            assert!(
                !route.csrf_protected,
                "GET {} should not require csrf",
                route.path
            );
        }
    }
}
