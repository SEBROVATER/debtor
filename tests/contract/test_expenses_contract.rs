use debtor::web::router::{RouteMethod, route_specs};

#[test]
fn expense_routes_match_contract() {
    let routes = route_specs();

    let expected = [
        (RouteMethod::Post, "/groups/{group_id}/expenses"),
        (
            RouteMethod::Patch,
            "/groups/{group_id}/expenses/{expense_id}",
        ),
        (
            RouteMethod::Delete,
            "/groups/{group_id}/expenses/{expense_id}",
        ),
    ];

    for (method, path) in expected {
        assert!(
            routes.iter().any(|r| r.method == method && r.path == path),
            "missing {method:?} {path}"
        );
    }
}
