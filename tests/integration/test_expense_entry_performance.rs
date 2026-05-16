use chrono::NaiveDate;
use chrono::Utc;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService};
use debtor::expenses::share_splitter::ShareInput;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Instant;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn expense_entry_meets_budget() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let group_service = debtor::groups::group_service::GroupService::new(state.db.clone());
    let member_service = debtor::groups::member_service::MemberService::new(state.db.clone());
    let expense_service = ExpenseService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = group_service
        .create_group("Trip", "USD", now)
        .await
        .expect("create group");

    let mut members = Vec::new();
    for idx in 0..5 {
        let member = member_service
            .add_member(&group.id, &format!("Member {idx}"), now)
            .await
            .expect("add member");
        members.push(member);
    }

    let shares = members
        .iter()
        .map(|m| ShareInput::equal(&m.id))
        .collect::<Vec<_>>();

    let start = Instant::now();
    let _ = expense_service
        .create_expense(
            CreateExpense {
                group_id: group.id.clone(),
                payer_member_id: members[0].id.clone(),
                amount: Decimal::from_str("50.00").unwrap(),
                currency: "USD".to_string(),
                expense_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                note: None,
                shares,
            },
            now,
        )
        .await
        .expect("create expense");
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 500, "took {:?}", elapsed);
}
