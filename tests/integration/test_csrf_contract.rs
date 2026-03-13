use debtor::web::csrf::{validate_csrf, CsrfError, CsrfToken};
use debtor::web::router::RouteMethod;

#[test]
fn csrf_is_required_for_state_changing_methods() {
    let token = CsrfToken::generate();

    assert!(validate_csrf(RouteMethod::Get, None, Some(&token)).is_ok());
    assert!(validate_csrf(RouteMethod::Post, Some(&token), Some(&token)).is_ok());

    let missing = validate_csrf(RouteMethod::Post, None, Some(&token));
    assert!(matches!(missing, Err(CsrfError::Missing)));

    let wrong = CsrfToken::generate();
    let mismatch = validate_csrf(RouteMethod::Patch, Some(&wrong), Some(&token));
    assert!(matches!(mismatch, Err(CsrfError::Mismatch)));
}
