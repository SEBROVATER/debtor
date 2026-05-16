use chrono::NaiveDate;
use chrono::Utc;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService, UpdateExpense};
use debtor::expenses::share_splitter::ShareInput;
use rust_decimal::Decimal;
use std::str::FromStr;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn updates_expense_and_shares() {
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
    let m2 = member_service
        .add_member(&group.id, "Bob", now)
        .await
        .expect("add member");

    let created = expense_service
        .create_expense(
            CreateExpense {
                group_id: group.id.clone(),
                payer_member_id: m1.id.clone(),
                amount: Decimal::from_str("100.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                note: Some("Dinner".to_string()),
                shares: vec![ShareInput::equal(&m1.id), ShareInput::equal(&m2.id)],
            },
            now,
        )
        .await
        .expect("create expense");

    let updated = expense_service
        .update_expense(
            &created.expense.id,
            UpdateExpense {
                payer_member_id: m2.id.clone(),
                amount: Decimal::from_str("120.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
                note: None,
                shares: vec![ShareInput::equal(&m1.id), ShareInput::equal(&m2.id)],
            },
            now,
        )
        .await
        .expect("update expense");

    assert_eq!(updated.expense.amount, Decimal::from_str("120.00").unwrap());
    assert_eq!(updated.shares.len(), 2);
}
