use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_flag_succeeds() {
  Command::cargo_bin("manta")
    .unwrap()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_version_flag_succeeds() {
  Command::cargo_bin("manta")
    .unwrap()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains("manta"));
}

#[test]
fn cli_invalid_subcommand_fails() {
  Command::cargo_bin("manta")
    .unwrap()
    .arg("nonexistent-command")
    .assert()
    .failure();
}

#[test]
fn cli_help_contains_site_flag() {
  Command::cargo_bin("manta")
    .unwrap()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("--site"));
}

#[test]
fn cli_get_sessions_help_succeeds() {
  Command::cargo_bin("manta")
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
  Command::cargo_bin("manta")
    .unwrap()
    .args(["--site", "mysite", "get", "sessions", "--help"])
    .assert()
    .success();
}

#[test]
fn cli_apply_boot_group_help_uses_group_name_placeholder() {
  Command::cargo_bin("manta")
    .unwrap()
    .args(["apply", "boot", "group", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("<GROUP_NAME>"))
    .stdout(predicate::str::contains("CLUSTER_NAME").not());
}

#[test]
fn cli_delete_configurations_accepts_dry_run() {
  Command::cargo_bin("manta")
    .unwrap()
    .args(["delete", "configurations", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn cli_delete_configurations_dry_run_short_alias() {
  Command::cargo_bin("manta")
    .unwrap()
    .args(["delete", "configurations", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("-d"));
}
