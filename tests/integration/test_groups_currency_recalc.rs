use chrono::Utc;
use debtor::groups::group_service::{GroupService, GroupUpdate};

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn currency_change_flags_refresh() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = GroupService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");

    let outcome = service
        .update_group(
            &group.id,
            GroupUpdate {
                name: None,
                target_currency: Some("EUR".to_string()),
            },
            now,
        )
        .await
        .expect("update group");

    assert!(outcome.currency_changed);
    assert_eq!(outcome.group.target_currency, "EUR");
}
