use std::fs;
use std::path::Path;

fn read_lower(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("read {}", path.display()))
        .to_lowercase()
}

#[test]
fn key_templates_have_semantic_structure() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let layout = read_lower(&root.join("src/web/templates/layout.html"));
    let login = read_lower(&root.join("src/web/templates/auth/login.html"));
    let group = read_lower(&root.join("src/web/templates/groups/detail.html"));
    let expenses = read_lower(&root.join("src/web/templates/expenses/list.html"));
    let debts = read_lower(&root.join("src/web/templates/debts/summary.html"));

    assert!(layout.contains("<header"), "layout missing header");
    assert!(layout.contains("<main"), "layout missing main");

    assert!(login.contains("<main"), "login missing main");
    assert!(login.contains("<form"), "login missing form");
    assert!(login.contains("<header"), "login missing header");

    assert!(group.contains("<main"), "group detail missing main");
    assert!(group.contains("<section"), "group detail missing sections");
    assert!(group.contains("<header"), "group detail missing header");

    assert!(expenses.contains("<section"), "expenses list missing section");
    assert!(expenses.contains("<header"), "expenses list missing header");
    assert!(expenses.contains("<ul"), "expenses list missing list");

    assert!(debts.contains("<section"), "debts summary missing section");
    assert!(debts.contains("debt-summary"), "debts summary missing class");
}
