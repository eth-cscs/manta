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
