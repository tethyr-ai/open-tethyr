//! Property 24: AX Version Validation and Handling

use open_tethyr::ax::AxValidator;

#[test]
fn version_1_0_accepted() {
    assert!(AxValidator::validate_version("1.0").is_ok());
}

#[test]
fn version_2_0_rejected() {
    assert!(AxValidator::validate_version("2.0").is_err());
}

#[test]
fn empty_version_rejected() {
    assert!(AxValidator::validate_version("").is_err());
}
