use debtor::web::router::route_specs;

#[test]
fn no_settlement_routes_exist() {
    let routes = route_specs();

    let forbidden = ["settle", "settlement", "pay", "payment"];
    for route in routes {
        for keyword in forbidden {
            assert!(
                !route.path.contains(keyword),
                "found forbidden route fragment `{keyword}` in {}",
                route.path
            );
        }
    }
}
