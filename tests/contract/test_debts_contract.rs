use debtor::web::router::{RouteMethod, route_specs};

#[test]
fn debt_summary_route_exists() {
    let routes = route_specs();
    assert!(
        routes
            .iter()
            .any(|r| r.method == RouteMethod::Get && r.path == "/groups/{group_id}/debts"),
        "missing GET /groups/{{group_id}}/debts"
    );
}
