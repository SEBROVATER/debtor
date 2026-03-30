use chrono::NaiveDate;
use chrono::Utc;
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::debts::debt_summary_service::DebtSummaryService;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService};
use debtor::expenses::share_splitter::ShareInput;
use debtor::groups::group_service::GroupService;
use debtor::groups::member_service::MemberService;
use rust_decimal::Decimal;
use std::str::FromStr;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn end_to_end_journey_from_login_to_summary() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let login_service = LoginService::new(state.db.clone());
    let group_service = GroupService::new(state.db.clone());
    let member_service = MemberService::new(state.db.clone());
    let expense_service = ExpenseService::new(state.db.clone());
    let summary_service = DebtSummaryService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let login = login_service
        .login("owner", "correct_password", now)
        .await
        .expect("login");

    match login {
        LoginResult::Success(_) => {}
        other => panic!("unexpected login result: {other:?}"),
    }

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

    expense_service
        .create_expense(
            CreateExpense {
                group_id: group.id.clone(),
                payer_member_id: alice.id.clone(),
                amount: Decimal::from_str("60.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                note: None,
                shares: vec![ShareInput::equal(&alice.id), ShareInput::equal(&bob.id)],
            },
            now,
        )
        .await
        .expect("create expense");

    let summary = summary_service
        .summarize_group(&group.id, now)
        .await
        .expect("summary");

    assert_eq!(summary.transfers.len(), 1);
    assert_eq!(summary.transfers[0].from_member_id, bob.id);
    assert_eq!(summary.transfers[0].to_member_id, alice.id);
    assert_eq!(
        summary.transfers[0].amount,
        Decimal::from_str("30.00").unwrap()
    );
}
