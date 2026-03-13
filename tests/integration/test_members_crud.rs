use chrono::Utc;
use debtor::groups::group_service::GroupService;
use debtor::groups::member_service::MemberService;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn member_crud_tracks_inactive_history() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let group_service = GroupService::new(state.db.clone());
    let member_service = MemberService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = group_service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");

    let member = member_service
        .add_member(&group.id, "Alice", now)
        .await
        .expect("add member");

    let renamed = member_service
        .rename_member(&group.id, &member.id, "Alice R", now)
        .await
        .expect("rename member");
    assert_eq!(renamed.display_name, "Alice R");

    member_service
        .remove_member(&group.id, &member.id, now)
        .await
        .expect("remove member");

    let active_only = member_service
        .list_members(&group.id, false)
        .await
        .expect("list active");
    assert!(active_only.is_empty(), "removed member should be inactive");

    let all_members = member_service
        .list_members(&group.id, true)
        .await
        .expect("list all");
    assert_eq!(all_members.len(), 1);
    assert!(!all_members[0].is_active);
    assert!(all_members[0].removed_at.is_some());
}
