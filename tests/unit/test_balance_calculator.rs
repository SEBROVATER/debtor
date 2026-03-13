use std::collections::HashMap;
use std::str::FromStr;

use debtor::debts::balance_calculator::{compute_balances, ExpenseShareSummary, MemberShare};
use rust_decimal::Decimal;

#[test]
fn aggregates_balances_across_expenses() {
    let expenses = vec![
        ExpenseShareSummary {
            payer_member_id: "a".to_string(),
            shares: vec![
                MemberShare {
                    member_id: "a".to_string(),
                    amount: Decimal::from_str("50.00").unwrap(),
                },
                MemberShare {
                    member_id: "b".to_string(),
                    amount: Decimal::from_str("50.00").unwrap(),
                },
            ],
        },
        ExpenseShareSummary {
            payer_member_id: "b".to_string(),
            shares: vec![
                MemberShare {
                    member_id: "a".to_string(),
                    amount: Decimal::from_str("20.00").unwrap(),
                },
                MemberShare {
                    member_id: "b".to_string(),
                    amount: Decimal::from_str("20.00").unwrap(),
                },
            ],
        },
    ];

    let balances = compute_balances(&expenses);
    let expected: HashMap<String, Decimal> = HashMap::from([
        ("a".to_string(), Decimal::from_str("30.00").unwrap()),
        ("b".to_string(), Decimal::from_str("-30.00").unwrap()),
    ]);

    assert_eq!(balances, expected);
}
