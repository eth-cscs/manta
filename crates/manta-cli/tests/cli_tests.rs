use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

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

/// Write a `cli.toml` with no `site` key into a fresh temp dir and
/// return the dir (kept alive so it isn't deleted) and the file path.
fn site_less_config() -> (tempfile::TempDir, std::path::PathBuf) {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("cli.toml");
  fs::write(
    &path,
    "log = \"info\"\n\
     manta_server_url = \"https://manta-server.example.com:8443\"\n",
  )
  .unwrap();
  (dir, path)
}

/// Bootstrap case: `config set site` must work even when no site is
/// configured yet — it only writes the local config, never reaching
/// the server.
#[test]
fn config_set_site_works_without_a_site_configured() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "set", "site", "mysite"])
    .assert()
    .success();
  let written = fs::read_to_string(&path).unwrap();
  assert!(
    written.contains("site = \"mysite\""),
    "config file should now carry the site key, got:\n{written}"
  );
}

/// `config show` is mostly local config, so it works with no site —
/// the site is reported as unset rather than erroring.
#[test]
fn config_show_works_without_a_site() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "show"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Current site: (unset)"))
    .stdout(predicate::str::contains(
      "Groups available: (no site selected)",
    ));
}

/// In JSON mode an unset site serializes to `null`, not an empty string.
#[test]
fn config_show_json_without_a_site_has_null_current_site() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "show", "--output", "json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"current_site\":null"));
}

/// A command that reaches the backend must fail fast with a clear
/// message when no site is selected — before any HTTP request.
#[test]
fn backend_command_without_a_site_errors_clearly() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["get", "groups"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("No site selected"));
}
