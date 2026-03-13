use std::str::FromStr;

use debtor::expenses::share_splitter::{ShareInput, ShareSplitError, normalize_shares};
use rust_decimal::Decimal;

#[test]
fn splits_equal_shares() {
    let total = Decimal::from_str("100.00").unwrap();
    let shares = vec![ShareInput::equal("m1"), ShareInput::equal("m2")];

    let result = normalize_shares(total, shares).expect("split");
    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0].computed_amount,
        Decimal::from_str("50.00").unwrap()
    );
    assert_eq!(
        result[1].computed_amount,
        Decimal::from_str("50.00").unwrap()
    );
}

#[test]
fn splits_percent_shares() {
    let total = Decimal::from_str("200.00").unwrap();
    let shares = vec![
        ShareInput::percent("m1", Decimal::from_str("25").unwrap()),
        ShareInput::percent("m2", Decimal::from_str("75").unwrap()),
    ];

    let result = normalize_shares(total, shares).expect("split");
    assert_eq!(
        result[0].computed_amount,
        Decimal::from_str("50.00").unwrap()
    );
    assert_eq!(
        result[1].computed_amount,
        Decimal::from_str("150.00").unwrap()
    );
}

#[test]
fn splits_mixed_amount_percent_and_equal() {
    let total = Decimal::from_str("100.00").unwrap();
    let shares = vec![
        ShareInput::amount("m1", Decimal::from_str("20.00").unwrap()),
        ShareInput::percent("m2", Decimal::from_str("25").unwrap()),
        ShareInput::equal("m3"),
        ShareInput::equal("m4"),
    ];

    let result = normalize_shares(total, shares).expect("split");

    let m1 = result.iter().find(|s| s.member_id == "m1").unwrap();
    let m2 = result.iter().find(|s| s.member_id == "m2").unwrap();
    let m3 = result.iter().find(|s| s.member_id == "m3").unwrap();
    let m4 = result.iter().find(|s| s.member_id == "m4").unwrap();

    assert_eq!(m1.computed_amount, Decimal::from_str("20.00").unwrap());
    assert_eq!(m2.computed_amount, Decimal::from_str("25.00").unwrap());
    assert_eq!(m3.computed_amount, Decimal::from_str("27.50").unwrap());
    assert_eq!(m4.computed_amount, Decimal::from_str("27.50").unwrap());
}

#[test]
fn rejects_percent_over_100() {
    let total = Decimal::from_str("100.00").unwrap();
    let shares = vec![
        ShareInput::percent("m1", Decimal::from_str("60").unwrap()),
        ShareInput::percent("m2", Decimal::from_str("60").unwrap()),
    ];

    let result = normalize_shares(total, shares);
    assert!(matches!(result, Err(ShareSplitError::InvalidPercentTotal)));
}
