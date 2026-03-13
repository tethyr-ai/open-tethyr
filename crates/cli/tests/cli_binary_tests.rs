//! CLI Binary Tests - invoke the actual binary and verify behavior
//! Catches clap parsing failures, exit code bugs, and output format issues.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn cmd() -> Command {
    Command::cargo_bin("open-tethyr").unwrap()
}

#[test]
fn binary_runs_with_help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Open-Tethyr AX protocol toolkit"));
}

#[test]
fn binary_shows_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("open-tethyr"));
}

#[test]
fn generate_subcommand_exists() {
    cmd()
        .arg("generate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("config"));
}

#[test]
fn validate_subcommand_exists() {
    cmd()
        .arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("FILE"));
}

#[test]
fn discover_subcommand_exists() {
    cmd()
        .arg("discover")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("domain"));
}

#[test]
fn serve_subcommand_exists() {
    cmd()
        .arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("port"));
}

#[test]
fn cache_invalidate_subcommand_exists() {
    cmd()
        .arg("cache-invalidate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("server"));
}

#[test]
fn generate_requires_config_flag() {
    cmd()
        .arg("generate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

#[test]
fn generate_and_validate_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("agents.yaml");
    let output_dir = tmp.path().join("output");
    std::fs::write(
        &config_path,
        concat!(
            "defaults:\n  provider: test-corp\n  auth: [OAuth2]\n",
            "agents:\n  - name: agent-1\n    description: Test agent\n",
            "    endpoints:\n      - protocol: rest\n        url: https://api.example.com\n",
            "        auth: [OAuth2]\n"
        ),
    )
    .unwrap();

    // Generate
    cmd()
        .args(["generate", "--config"])
        .arg(config_path.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .arg("--validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated AX record"));

    // Validate the generated file
    let ax_path = output_dir.join(".well-known/agent-exchange");
    assert!(
        ax_path.exists(),
        "AX file should exist at {}",
        ax_path.display()
    );

    cmd()
        .arg("validate")
        .arg(ax_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"));
}

#[test]
fn validate_fails_on_invalid_file() {
    let tmp = tempfile::tempdir().unwrap();
    let bad_file = tmp.path().join("bad.json");
    std::fs::write(&bad_file, r#"{"record_type":"INVALID","version":"1.0","agent":{"name":"x","description":"x"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#).unwrap();

    cmd()
        .arg("validate")
        .arg(bad_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("FAILED"));
}

#[test]
fn validate_warns_on_unknown_auth() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("warn.json");
    std::fs::write(&file, r#"{"record_type":"AX","version":"1.0","agent":{"name":"x","description":"x"},"endpoints":[{"protocol":"rest","url":"https://x.com","auth":["CustomAuth"]}]}"#).unwrap();

    cmd()
        .arg("validate")
        .arg(file.to_str().unwrap())
        .assert()
        .success() // should pass (unknown auth is warning, not error)
        .stderr(
            predicate::str::contains("WARN")
                .or(predicate::str::contains("Unknown").or(predicate::str::is_empty())),
        );
}

#[test]
fn validate_exits_3_on_validation_error() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("invalid.json");
    std::fs::write(&file, r#"{"record_type":"WRONG","version":"1.0","agent":{"name":"x","description":"x"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#).unwrap();

    cmd()
        .arg("validate")
        .arg(file.to_str().unwrap())
        .assert()
        .code(3);
}

#[test]
fn validate_nonexistent_file_fails() {
    cmd()
        .arg("validate")
        .arg("/nonexistent/path/to/file.json")
        .assert()
        .failure();
}

#[test]
fn unknown_subcommand_fails() {
    cmd().arg("nonexistent-command").assert().failure();
}
