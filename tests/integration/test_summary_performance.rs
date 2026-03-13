use chrono::{NaiveDate, Utc};
use debtor::debts::debt_summary_service::DebtSummaryService;
use debtor::expenses::expense_service::{CreateExpense, ExpenseService};
use debtor::expenses::share_splitter::ShareInput;
use debtor::groups::group_service::GroupService;
use debtor::groups::member_service::MemberService;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Instant;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn debt_summary_meets_large_group_budget() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let group_service = GroupService::new(state.db.clone());
    let member_service = MemberService::new(state.db.clone());
    let expense_service = ExpenseService::new(state.db.clone());
    let summary_service = DebtSummaryService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let group = group_service
        .create_group("Summit", "USD", now)
        .await
        .expect("create group");

    let mut members = Vec::new();
    for idx in 0..20 {
        let member = member_service
            .add_member(&group.id, &format!("Member {idx}"), now)
            .await
            .expect("add member");
        members.push(member);
    }

    let share_template = members
        .iter()
        .map(|member| ShareInput::equal(&member.id))
        .collect::<Vec<_>>();

    let expense_date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    for _idx in 0..200 {
        let shares = share_template.clone();
        let payer = &members[0];
        expense_service
            .create_expense(
                CreateExpense {
                    group_id: group.id.clone(),
                    payer_member_id: payer.id.clone(),
                    amount: Decimal::from_str("120.00").unwrap(),
                    currency: "USD".to_string(),
                    expense_date,
                    note: None,
                    shares,
                },
                now,
            )
            .await
            .expect("create expense");
    }

    let start = Instant::now();
    let summary = summary_service
        .summarize_group(&group.id, now)
        .await
        .expect("summary");
    let elapsed = start.elapsed();

    assert!(!summary.transfers.is_empty());
    assert!(elapsed.as_millis() < 3000, "took {:?}", elapsed);
}
