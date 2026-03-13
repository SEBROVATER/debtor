use debtor::web::router::{route_specs, RouteMethod};

#[test]
fn member_routes_match_contract() {
    let routes = route_specs();

    let expected = [
        (RouteMethod::Post, "/groups/{group_id}/members"),
        (RouteMethod::Patch, "/groups/{group_id}/members/{member_id}"),
        (RouteMethod::Delete, "/groups/{group_id}/members/{member_id}"),
    ];

    for (method, path) in expected {
        assert!(
            routes.iter().any(|r| r.method == method && r.path == path),
            "missing {method:?} {path}"
        );
    }
}
