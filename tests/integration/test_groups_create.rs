use chrono::Utc;
use debtor::groups::group_service::GroupService;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn creates_and_lists_groups() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = GroupService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let created = service
        .create_group("Ski Trip", "USD", now)
        .await
        .expect("create group");

    let groups = service.list_groups().await.expect("list groups");
    assert!(groups.iter().any(|g| g.id == created.id));
}
