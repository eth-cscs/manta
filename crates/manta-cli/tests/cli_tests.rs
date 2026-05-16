use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_flag_succeeds() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_version_flag_succeeds() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains("manta-cli"));
}

#[test]
fn cli_invalid_subcommand_fails() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .arg("nonexistent-command")
    .assert()
    .failure();
}

#[test]
fn cli_help_contains_site_flag() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("--site"));
}

#[test]
fn cli_get_sessions_help_succeeds() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .args(["get", "sessions", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--hsm-group"))
    .stdout(predicate::str::contains("--limit"))
    .stdout(predicate::str::contains("--most-recent"));
}

#[test]
fn cli_site_flag_accepted_with_help() {
  Command::cargo_bin("manta-cli")
    .unwrap()
    .args(["--site", "mysite", "get", "sessions", "--help"])
    .assert()
    .success();
}
