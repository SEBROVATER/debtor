use debtor::web::router::{route_specs, RouteMethod};

#[test]
fn foundation_routes_include_health_login_and_dashboard() {
    let routes = route_specs();

    let health = routes.iter().find(|r| r.method == RouteMethod::Get && r.path == "/health");
    assert!(health.is_some(), "expected /health route");
    assert!(!health.unwrap().requires_auth, "health should be public");

    let login_get = routes.iter().find(|r| r.method == RouteMethod::Get && r.path == "/login");
    assert!(login_get.is_some(), "expected GET /login");
    assert!(!login_get.unwrap().requires_auth, "login should be public");

    let dashboard = routes.iter().find(|r| r.method == RouteMethod::Get && r.path == "/dashboard");
    assert!(dashboard.is_some(), "expected /dashboard route");
    assert!(dashboard.unwrap().requires_auth, "dashboard should be protected");
}
