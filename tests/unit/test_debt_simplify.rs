use std::str::FromStr;

use debtor::debts::simplify::minimal_transfers;
use rust_decimal::Decimal;

#[test]
fn simplifies_to_minimal_transfers() {
    let balances = vec![
        ("a".to_string(), Decimal::from_str("10.00").unwrap()),
        ("b".to_string(), Decimal::from_str("-10.00").unwrap()),
    ];

    let transfers = minimal_transfers(&balances);
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].from_member_id, "b");
    assert_eq!(transfers[0].to_member_id, "a");
    assert_eq!(transfers[0].amount, Decimal::from_str("10.00").unwrap());
}

#[test]
fn partitions_into_zero_sum_sets() {
    let balances = vec![
        ("a".to_string(), Decimal::from_str("5.00").unwrap()),
        ("b".to_string(), Decimal::from_str("5.00").unwrap()),
        ("c".to_string(), Decimal::from_str("-5.00").unwrap()),
        ("d".to_string(), Decimal::from_str("-5.00").unwrap()),
    ];

    let transfers = minimal_transfers(&balances);
    assert_eq!(transfers.len(), 2);
}
