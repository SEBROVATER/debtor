use chrono::NaiveDateTime;
use debtor::exchange_rates::frankfurter_client::FrankfurterClient;

#[test]
fn maps_frankfurter_response() {
    let body = r#"{
  "amount": 1.0,
  "base": "USD",
  "date": "2026-02-23",
  "rates": {"EUR": 0.92, "PLN": 3.95}
}"#;

    let fetched_at =
        NaiveDateTime::parse_from_str("2026-02-24 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let quotes = FrankfurterClient::parse_response(body, fetched_at).expect("parse response");

    assert_eq!(quotes.len(), 2);
    let eur = quotes.iter().find(|q| q.to_currency == "EUR").unwrap();
    assert_eq!(eur.from_currency, "USD");
    assert_eq!(eur.rate.to_string(), "0.92");
    assert_eq!(eur.rate_date.to_string(), "2026-02-23");
    assert_eq!(eur.provider, "frankfurter");
}
