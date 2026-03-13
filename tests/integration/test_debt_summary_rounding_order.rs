use chrono::NaiveDate;
use chrono::Utc;
use debtor::debts::debt_summary_service::DebtSummaryService;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService};
use debtor::expenses::share_splitter::ShareInput;
use rust_decimal::Decimal;
use std::str::FromStr;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn summary_is_deterministic_and_rounded() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let group_service = debtor::groups::group_service::GroupService::new(state.db.clone());
    let member_service = debtor::groups::member_service::MemberService::new(state.db.clone());
    let expense_service = ExpenseService::new(state.db.clone());
    let summary_service = DebtSummaryService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = group_service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");
    let alice = member_service
        .add_member(&group.id, "Alice", now)
        .await
        .expect("add member");
    let bob = member_service
        .add_member(&group.id, "Bob", now)
        .await
        .expect("add member");
    let carol = member_service
        .add_member(&group.id, "Carol", now)
        .await
        .expect("add member");

    expense_service
        .create_expense(
            CreateExpense {
                group_id: group.id.clone(),
                payer_member_id: alice.id.clone(),
                amount: Decimal::from_str("10.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                note: None,
                shares: vec![
                    ShareInput::percent(&alice.id, Decimal::from_str("33.3333").unwrap()),
                    ShareInput::percent(&bob.id, Decimal::from_str("33.3333").unwrap()),
                    ShareInput::percent(&carol.id, Decimal::from_str("33.3334").unwrap()),
                ],
            },
            now,
        )
        .await
        .expect("create expense");

    let summary = summary_service
        .summarize_group(&group.id, now)
        .await
        .expect("summary");

    assert_eq!(summary.transfers.len(), 2);
    assert!(summary.transfers[0].amount.round_dp(2) == summary.transfers[0].amount);
    assert!(summary.transfers[1].amount.round_dp(2) == summary.transfers[1].amount);

    let ordered = summary
        .transfers
        .windows(2)
        .all(|pair| pair[0].from_member_id <= pair[1].from_member_id);
    assert!(ordered);
}
