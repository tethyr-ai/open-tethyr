//! Property 25: Auth Method Validation

use open_tethyr::error::{is_valid_auth_method, APPROVED_AUTH_METHODS};

#[test]
fn approved_methods_accepted() {
    for method in APPROVED_AUTH_METHODS {
        assert!(is_valid_auth_method(method), "Should accept {}", method);
    }
}

#[test]
fn invalid_methods_rejected() {
    assert!(!is_valid_auth_method("INVALID"));
    assert!(!is_valid_auth_method("basic"));
    assert!(!is_valid_auth_method("Bearer"));
    assert!(!is_valid_auth_method(""));
}
