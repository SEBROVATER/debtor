use chrono::NaiveDate;
use chrono::Utc;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService};
use debtor::expenses::share_splitter::ShareInput;
use rust_decimal::Decimal;
use std::str::FromStr;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn deletes_expense() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let group_service = debtor::groups::group_service::GroupService::new(state.db.clone());
    let member_service = debtor::groups::member_service::MemberService::new(state.db.clone());
    let expense_service = ExpenseService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = group_service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");
    let m1 = member_service
        .add_member(&group.id, "Alice", now)
        .await
        .expect("add member");

    let created = expense_service
        .create_expense(
            CreateExpense {
                group_id: group.id.clone(),
                payer_member_id: m1.id.clone(),
                amount: Decimal::from_str("25.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                note: None,
                shares: vec![ShareInput::equal(&m1.id)],
            },
            now,
        )
        .await
        .expect("create expense");

    let deleted = expense_service
        .delete_expense(&created.expense.id)
        .await
        .expect("delete expense");
    assert!(deleted);

    let remaining = expense_service
        .list_expenses(&group.id)
        .await
        .expect("list expenses");
    assert!(remaining.is_empty());
}
