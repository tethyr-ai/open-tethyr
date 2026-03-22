//! Property 20: Configuration Validation Correctness

use open_tethyr::config::ConfigValidator;

#[test]
fn valid_domain_passes() {
    assert!(ConfigValidator::validate_domain("example.com").is_ok());
    assert!(ConfigValidator::validate_domain("sub.example.com").is_ok());
    assert!(ConfigValidator::validate_domain("localhost").is_ok());
}

#[test]
fn invalid_domain_fails() {
    assert!(ConfigValidator::validate_domain("").is_err());
    assert!(ConfigValidator::validate_domain("-bad.com").is_err());
    assert!(ConfigValidator::validate_domain("bad..com").is_err());
}

#[test]
fn valid_port_passes() {
    assert!(ConfigValidator::validate_port(80).is_ok());
    assert!(ConfigValidator::validate_port(443).is_ok());
    assert!(ConfigValidator::validate_port(8080).is_ok());
    assert!(ConfigValidator::validate_port(65535).is_ok());
}

#[test]
fn invalid_port_fails() {
    assert!(ConfigValidator::validate_port(0).is_err());
    assert!(ConfigValidator::validate_port(65536).is_err());
}

#[test]
fn valid_url_passes() {
    assert!(ConfigValidator::validate_url("https://example.com").is_ok());
}

#[test]
fn http_url_fails() {
    assert!(ConfigValidator::validate_url("http://example.com").is_err());
    assert!(ConfigValidator::validate_url("").is_err());
}

#[test]
fn valid_ttl_passes() {
    assert_eq!(ConfigValidator::validate_ttl("3600").unwrap(), 3600);
    assert_eq!(ConfigValidator::validate_ttl("1").unwrap(), 1);
}

#[test]
fn invalid_ttl_fails() {
    assert!(ConfigValidator::validate_ttl("0").is_err());
    assert!(ConfigValidator::validate_ttl("-1").is_err());
    assert!(ConfigValidator::validate_ttl("abc").is_err());
}
