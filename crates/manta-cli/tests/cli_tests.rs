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
/// the site is reported as unset rather than erroring, and both the
/// JWT and server sections render the "unavailable" marker.
#[test]
fn config_show_works_without_a_site() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "show"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Current site:"))
    .stdout(predicate::str::contains("(unset)"))
    .stdout(predicate::str::contains("From JWT token:"))
    .stdout(predicate::str::contains("From server API:"))
    .stdout(predicate::str::contains("(unavailable — no site selected)"));
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

/// `config set read-only` writes `read_only = true` into cli.toml —
/// works with no site configured, no server round-trip.
#[test]
fn config_set_read_only_writes_true_to_cli_toml() {
  let (_dir, path) = site_less_config();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "set", "read-only"])
    .assert()
    .success();
  let written = fs::read_to_string(&path).unwrap();
  assert!(
    written.contains("read_only = true"),
    "config file should now carry read_only = true, got:\n{written}"
  );
}

/// `config unset read-only` removes the key from cli.toml — inverse
/// of `config set read-only`.
#[test]
fn config_unset_read_only_removes_the_key() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("cli.toml");
  fs::write(
    &path,
    "log = \"info\"\n\
     manta_server_url = \"https://manta-server.example.com:8443\"\n\
     read_only = true\n",
  )
  .unwrap();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["config", "unset", "read-only"])
    .assert()
    .success();
  let written = fs::read_to_string(&path).unwrap();
  assert!(
    !written.contains("read_only"),
    "read_only key should be gone, got:\n{written}"
  );
}

/// With `read_only = true` in cli.toml, a mutating verb is refused
/// locally — before any HTTP request. Message points at
/// `manta config unset read-only`.
#[test]
fn read_only_gate_refuses_mutating_verb_locally() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("cli.toml");
  fs::write(
    &path,
    "log = \"info\"\n\
     site = \"alps\"\n\
     manta_server_url = \"https://manta-server.example.com:8443\"\n\
     read_only = true\n",
  )
  .unwrap();
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["delete", "session", "any-session-id"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("read-only mode"))
    .stderr(predicate::str::contains("manta config unset read-only"));
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

// ---------------------------------------------------------------------------
// CLI-side site pre-resolution via manta-cache (cache_url in cli.toml)
// ---------------------------------------------------------------------------

/// Write a `cli.toml` with no `site` but a `cache_url`, plus
/// `read_only = true` so mutating commands stop deterministically
/// (offline, no auth prompt) right after resolution.
fn cache_config(
  cache_url: &str,
  read_only: bool,
) -> (tempfile::TempDir, std::path::PathBuf) {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("cli.toml");
  fs::write(
    &path,
    format!(
      "log = \"error\"\n\
       manta_server_url = \"https://127.0.0.1:1\"\n\
       cache_url = \"{cache_url}\"\n\
       read_only = {read_only}\n"
    ),
  )
  .unwrap();
  (dir, path)
}

/// One-shot HTTP mock: accepts a single connection and answers with
/// `body` as JSON. Returns the base URL to put in `cache_url`.
fn spawn_cache_mock(body: &'static str) -> String {
  let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let addr = listener.local_addr().unwrap();
  std::thread::spawn(move || {
    if let Ok((mut stream, _)) = listener.accept() {
      use std::io::{Read, Write};
      let mut buf = [0u8; 4096];
      let _ = stream.read(&mut buf);
      let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
      );
    }
  });
  format!("http://{addr}")
}

/// Scenario 1: a group-targeting command with no site resolves the
/// site through the cache (visible on stderr), then proceeds — here
/// into the read-only gate, proving resolution happened first and the
/// "No site selected" path was never taken.
#[test]
fn cache_resolves_site_for_group_command() {
  let url = spawn_cache_mock(r#"{"site":"alps"}"#);
  let (_dir, path) = cache_config(&url, true);
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["power", "off", "group", "compute", "--assume-yes"])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
      "site 'alps' resolved via manta-cache (group 'compute')",
    ))
    .stderr(predicate::str::contains("read-only mode"))
    .stderr(predicate::str::contains("No site selected").not());
}

/// Scenario 6 (cache down): resolution degrades with a warning and the
/// command falls back to today's "No site selected" error.
#[test]
fn cache_unreachable_degrades_to_site_required() {
  // Bind then immediately drop the listener so the port is closed.
  let addr = {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
  };
  let (_dir, path) = cache_config(&format!("http://{addr}"), false);
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["get", "group-nodes", "compute"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("site resolution degraded"))
    .stderr(predicate::str::contains("No site selected"));
}

/// Scenario 3 (split xname list): the cache answers, the answer names
/// two sites, and the CLI aborts with the per-xname resolutions —
/// before any manta-server call.
#[test]
fn cache_split_xnames_error_names_both_sites() {
  let url = spawn_cache_mock(
    r#"{"site":null,
        "resolutions":{"x1000c0s0b0n0":"alps","x2000c0s0b0n0":"daint"},
        "unknown":[]}"#,
  );
  let (_dir, path) = cache_config(&url, false);
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .args(["get", "nodes", "x1000c0s0b0n0,x2000c0s0b0n0"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("do not resolve to a single site"))
    .stderr(predicate::str::contains("x1000c0s0b0n0 → alps"))
    .stderr(predicate::str::contains("x2000c0s0b0n0 → daint"));
}
