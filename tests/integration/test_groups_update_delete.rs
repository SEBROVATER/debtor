use chrono::Utc;
use debtor::groups::group_service::GroupService;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn group_delete_removes_group() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = GroupService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");

    let deleted = service
        .delete_group(&group.id)
        .await
        .expect("delete group");
    assert!(deleted);

    let groups = service.list_groups().await.expect("list groups");
    assert!(groups.is_empty());
}
