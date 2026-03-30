use debtor::web::router::{RouteMethod, route_specs};

#[test]
fn auth_routes_exist_and_registration_is_absent() {
    let routes = route_specs();

    assert!(
        routes
            .iter()
            .any(|r| r.method == RouteMethod::Get && r.path == "/login"),
        "missing GET /login"
    );
    assert!(
        routes
            .iter()
            .any(|r| r.method == RouteMethod::Post && r.path == "/login"),
        "missing POST /login"
    );
    assert!(
        routes
            .iter()
            .any(|r| r.method == RouteMethod::Post && r.path == "/logout"),
        "missing POST /logout"
    );
    assert!(
        !routes.iter().any(|r| r.path.starts_with("/register")),
        "registration route should not exist"
    );
}
