use debtor::web::router::{route_specs, RouteMethod};

#[test]
fn groups_routes_match_contract() {
    let routes = route_specs();

    let expected = [
        (RouteMethod::Post, "/groups"),
        (RouteMethod::Get, "/groups/{group_id}"),
        (RouteMethod::Patch, "/groups/{group_id}"),
        (RouteMethod::Delete, "/groups/{group_id}"),
    ];

    for (method, path) in expected {
        assert!(
            routes.iter().any(|r| r.method == method && r.path == path),
            "missing {method:?} {path}"
        );
    }
}
