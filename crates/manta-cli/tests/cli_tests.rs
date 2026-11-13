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

/// A command whose endpoint cannot resolve the site must fail fast
/// with a clear message when no site is selected — before any request
/// to manta-server. (The service account logs in at a mock issuer;
/// the login needs no site.)
#[test]
fn backend_command_without_a_site_errors_clearly() {
  let (_dir, path) = site_less_config_for(
    "https://manta-server.example.com:8443",
    &spawn_oidc_mock(),
  );
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .env("MANTA_SA_CLIENT_ID", "test-client")
    .env("MANTA_SA_CLIENT_SECRET", "test-secret")
    .args(["get", "groups"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("No site selected"));
}

// ---------------------------------------------------------------------------
// Server-side site resolution: commands that send no X-Manta-Site
// ---------------------------------------------------------------------------

/// Minimal sequential HTTP mock of manta-server: answers the `i`-th
/// connection with `responses[i]` (`(status line, extra header lines,
/// JSON body)`) and reports each request's head (request line +
/// headers, lower-cased names as reqwest sends them) on the channel.
fn spawn_manta_mock(
  responses: Vec<(&'static str, &'static str, &'static str)>,
) -> (String, std::sync::mpsc::Receiver<String>) {
  use std::io::{BufRead, BufReader, Read, Write};
  let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let addr = listener.local_addr().unwrap();
  let (tx, rx) = std::sync::mpsc::channel();
  std::thread::spawn(move || {
    for (status, headers, body) in responses {
      let Ok((stream, _)) = listener.accept() else {
        return;
      };
      let mut reader = BufReader::new(stream);
      let mut head = String::new();
      let mut content_length = 0usize;
      loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
          break;
        }
        if let Some(v) =
          line.to_ascii_lowercase().strip_prefix("content-length:")
        {
          content_length = v.trim().parse().unwrap_or(0);
        }
        head.push_str(&line);
      }
      let mut req_body = vec![0u8; content_length];
      let _ = reader.read_exact(&mut req_body);
      let _ = tx.send(head);
      let mut stream = reader.into_inner();
      let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{headers}\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
      );
    }
  });
  (format!("http://{addr}"), rx)
}

/// Mock OIDC issuer for the service-account login: serves the
/// discovery document and grants a token to any client-credentials
/// request, for as long as the test runs (the CLI fetches a new token
/// for every request). Returns the issuer URL.
fn spawn_oidc_mock() -> String {
  use std::io::{BufRead, BufReader, Read, Write};
  let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let issuer = format!("http://{}", listener.local_addr().unwrap());
  let discovery = format!(r#"{{"token_endpoint":"{issuer}/token"}}"#);
  std::thread::spawn(move || {
    for stream in listener.incoming() {
      let Ok(stream) = stream else { return };
      let mut reader = BufReader::new(stream);
      let mut request_line = String::new();
      let _ = reader.read_line(&mut request_line);
      let mut content_length = 0usize;
      loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
          break;
        }
        if let Some(v) =
          line.to_ascii_lowercase().strip_prefix("content-length:")
        {
          content_length = v.trim().parse().unwrap_or(0);
        }
      }
      let mut req_body = vec![0u8; content_length];
      let _ = reader.read_exact(&mut req_body);
      let body = if request_line.starts_with("GET ") {
        discovery.clone()
      } else {
        r#"{"access_token":"test-token","expires_in":300}"#.to_string()
      };
      let _ = write!(
        reader.into_inner(),
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
      );
    }
  });
  issuer
}

/// `cli.toml` with no `site`, pointing at `server_url`, logging in at
/// `issuer_url`.
fn site_less_config_for(
  server_url: &str,
  issuer_url: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("cli.toml");
  fs::write(
    &path,
    format!(
      "log = \"error\"\nmanta_server_url = \"{server_url}\"\n\
       oidc_issuer_url = \"{issuer_url}\"\n"
    ),
  )
  .unwrap();
  (dir, path)
}

/// `power` without a site: the POST goes out without `X-Manta-Site`
/// but with a token, the CLI adopts the site the server echoes, and the
/// transition poll (whose endpoint cannot resolve anything) carries it.
#[test]
fn power_without_a_site_pins_the_site_the_server_resolved() {
  let (url, requests) = spawn_manta_mock(vec![
    (
      "200 OK",
      "X-Manta-Site: alps\r\n",
      r#"{"transitionID":"t-1","operation":"off"}"#,
    ),
    (
      "200 OK",
      "",
      r#"{"transitionStatus":"completed","taskCounts":{"failed":0}}"#,
    ),
  ]);
  let (_dir, path) = site_less_config_for(&url, &spawn_oidc_mock());
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .env("MANTA_SA_CLIENT_ID", "test-client")
    .env("MANTA_SA_CLIENT_SECRET", "test-secret")
    .args(["power", "off", "group", "compute", "--assume-yes"])
    .assert()
    .success()
    .stderr(predicate::str::contains(
      "site 'alps' resolved by manta-server",
    ));

  let post = requests.recv().unwrap();
  assert!(post.starts_with("POST /v2/power "), "{post}");
  assert!(!post.contains("x-manta-site"), "{post}");
  // No site is no reason to go unauthenticated: manta-server needs the
  // token to resolve the site and to exchange it there.
  assert!(post.contains("authorization: Bearer test-token"), "{post}");
  let poll = requests.recv().unwrap();
  assert!(poll.starts_with("GET /v2/power/transitions/t-1 "), "{poll}");
  assert!(poll.contains("x-manta-site: alps"), "{poll}");
  assert!(poll.contains("authorization: Bearer test-token"), "{poll}");
}

/// A resolution failure is the server's to explain; the CLI shows its
/// message.
#[test]
fn server_resolution_error_is_shown_verbatim() {
  let (url, _requests) = spawn_manta_mock(vec![(
    "409 Conflict",
    "",
    r#"{"error":"The xnames do not resolve to a single site (x1000c0s0b0n0 → alps; x2000c0s0b0n0 → daint). Split the command per site or pass --site <name> explicitly."}"#,
  )]);
  let (_dir, path) = site_less_config_for(&url, &spawn_oidc_mock());
  Command::cargo_bin("manta")
    .unwrap()
    .env("MANTA_CLI_CONFIG", &path)
    .env("MANTA_SA_CLIENT_ID", "test-client")
    .env("MANTA_SA_CLIENT_SECRET", "test-secret")
    .args(["get", "nodes", "x1000c0s0b0n0,x2000c0s0b0n0"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("HTTP 409"))
    .stderr(predicate::str::contains("x2000c0s0b0n0 → daint"));
}
