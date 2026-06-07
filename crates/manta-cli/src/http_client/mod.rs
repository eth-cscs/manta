//! Thin HTTP client for forwarding CLI calls to the manta server.
//!
//! Every CLI command goes through this client; the server resolves CA
//! certs, base URLs, and credentials internally — the CLI only sends
//! `X-Manta-Site` + `Authorization: Bearer <token>`.
//!
//! The endpoint methods are split per-resource across sub-modules
//! (mirroring `crates/manta-server/src/server/handlers/`) so each
//! file covers one slice of the HTTP API. All methods are still
//! `pub fn`s on the single `MantaClient` struct — the split is
//! purely organisational: external callers (`client.get_sessions(...)`,
//! `client.add_node(...)`, …) don't change.

mod client;
mod query;
mod wire;

mod auth;
mod boot_parameters;
mod configurations;
mod console;
mod ephemeral_env;
mod groups;
mod hardware_clusters;
mod images;
mod kernel_parameters;
mod migrate;
mod nodes;
mod power;
mod redfish_endpoints;
mod sat_file;
mod sessions;
mod templates;
mod vcluster;

pub use auth::AuthServerUnreachable;
pub use client::MantaClient;
pub(super) use query::QueryBuilder;
pub(super) use wire::ws_base_url;

pub use boot_parameters::ApplyBootConfigRequest;
pub use hardware_clusters::ApplyHwConfigurationRequest;
pub use kernel_parameters::{
  AddKernelParametersRequest, ApplyKernelParametersRequest,
};
pub use sat_file::CreateImageCfsSessionRequest;
pub use sessions::CreateSessionRequest;
pub use templates::ApplyTemplateSessionRequest;
pub use vcluster::RestoreVclusterRequest;

#[cfg(test)]
mod tests {
  use super::QueryBuilder;
  use super::wire::{redact_json_secrets, ws_base_url};

  // ---- ws_base_url ----

  #[test]
  fn ws_base_url_promotes_https_to_wss() {
    assert_eq!(
      ws_base_url("https://manta.example:8443"),
      "wss://manta.example:8443"
    );
  }

  #[test]
  fn ws_base_url_promotes_http_to_ws() {
    assert_eq!(ws_base_url("http://localhost:8080"), "ws://localhost:8080");
  }

  #[test]
  fn ws_base_url_passes_through_unknown_scheme() {
    // Defensive: should never receive a non-HTTP URL, but if it does
    // we hand it back untouched rather than mangling it.
    assert_eq!(ws_base_url("ftp://example"), "ftp://example");
    assert_eq!(ws_base_url(""), "");
  }

  #[test]
  fn ws_base_url_preserves_path_and_query() {
    assert_eq!(
      ws_base_url("https://h.example/api/v1?x=1"),
      "wss://h.example/api/v1?x=1"
    );
  }

  // ---- redact_json_secrets ----

  #[test]
  fn redact_replaces_password_and_token_at_top_level() {
    let body = r#"{"username":"alice","password":"hunter2"}"#;
    let out = redact_json_secrets(body);
    assert!(out.contains("\"username\":\"alice\""));
    assert!(out.contains("\"password\":\"<REDACTED>\""));
    assert!(!out.contains("hunter2"));
  }

  #[test]
  fn redact_replaces_token_field() {
    let body = r#"{"token":"eyJhbGciOi..."}"#;
    let out = redact_json_secrets(body);
    assert!(out.contains("\"token\":\"<REDACTED>\""));
    assert!(!out.contains("eyJ"));
  }

  #[test]
  fn redact_walks_into_nested_objects() {
    let body = r#"{"outer":{"password":"x"},"inner":{"deep":{"token":"y"}}}"#;
    let out = redact_json_secrets(body);
    assert!(!out.contains("\"x\""));
    assert!(!out.contains("\"y\""));
    assert_eq!(out.matches("<REDACTED>").count(), 2);
  }

  #[test]
  fn redact_walks_through_arrays() {
    let body = r#"{"creds":[{"password":"a"},{"password":"b"}]}"#;
    let out = redact_json_secrets(body);
    assert!(!out.contains("\"a\""));
    assert!(!out.contains("\"b\""));
    assert_eq!(out.matches("<REDACTED>").count(), 2);
  }

  #[test]
  fn redact_leaves_unrelated_fields_alone() {
    let body = r#"{"a":1,"b":"x","c":{"d":[1,2,3]}}"#;
    let out = redact_json_secrets(body);
    // Round-trips structurally; just verify nothing got <REDACTED>.
    assert!(!out.contains("<REDACTED>"));
  }

  #[test]
  fn redact_passes_through_non_json_unchanged() {
    let body = "plain text body";
    assert_eq!(redact_json_secrets(body), "plain text body");
  }

  // ---- QueryBuilder ----

  #[test]
  fn query_builder_empty_returns_no_pairs() {
    assert!(QueryBuilder::new().build().is_empty());
  }

  #[test]
  fn query_builder_opt_skips_none_includes_some() {
    let q = QueryBuilder::new()
      .opt("a", &None)
      .opt("b", &Some("x".to_string()))
      .build();
    assert_eq!(q, vec![("b", "x".to_string())]);
  }

  #[test]
  fn query_builder_opt_display_renders_numeric_values() {
    let q = QueryBuilder::new().opt_display("limit", &Some(7u8)).build();
    assert_eq!(q, vec![("limit", "7".to_string())]);
  }

  #[test]
  fn query_builder_vec_joins_with_comma_when_non_empty() {
    let q = QueryBuilder::new()
      .vec("xnames", &["a".to_string(), "b".to_string()])
      .build();
    assert_eq!(q, vec![("xnames", "a,b".to_string())]);
  }

  #[test]
  fn query_builder_vec_skips_when_empty() {
    let q = QueryBuilder::new().vec("xnames", &[]).build();
    assert!(q.is_empty());
  }

  #[test]
  fn query_builder_flag_only_pushes_when_true() {
    let q = QueryBuilder::new()
      .flag("dry_run", true)
      .flag("force", false)
      .build();
    assert_eq!(q, vec![("dry_run", "true".to_string())]);
  }

  #[test]
  fn query_builder_pair_always_pushes() {
    let q = QueryBuilder::new()
      .pair("ids", "a,b,c".to_string())
      .pair("dry_run", "false".to_string())
      .build();
    assert_eq!(
      q,
      vec![
        ("ids", "a,b,c".to_string()),
        ("dry_run", "false".to_string()),
      ]
    );
  }

  #[test]
  fn query_builder_preserves_insertion_order() {
    let q = QueryBuilder::new()
      .pair("a", "1".into())
      .pair("b", "2".into())
      .pair("c", "3".into())
      .build();
    assert_eq!(
      q.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
      vec!["a", "b", "c"]
    );
  }

  #[test]
  fn query_builder_chains_mixed_methods() {
    let q = QueryBuilder::new()
      .opt("site", &Some("alps".into()))
      .opt("group", &None)
      .vec(
        "xnames",
        &["x3000c0s1b0n0".to_string(), "x3000c0s2b0n0".to_string()],
      )
      .flag("output_pretty", true)
      .build();
    assert_eq!(
      q,
      vec![
        ("site", "alps".to_string()),
        ("xnames", "x3000c0s1b0n0,x3000c0s2b0n0".to_string()),
        ("output_pretty", "true".to_string()),
      ]
    );
  }

  // ---- MantaClient timeout constructors ----

  use super::MantaClient;

  #[test]
  fn new_with_timeout_none_pins_url_and_site() {
    let c = MantaClient::new_with_timeout("http://stub.invalid", "alps", None)
      .expect("construction with timeout=None must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid/api/v1");
    assert_eq!(c.site_name(), "alps");
  }

  #[test]
  fn new_with_timeout_some_pins_url_and_site() {
    let c =
      MantaClient::new_with_timeout("http://stub.invalid", "alps", Some(5))
        .expect("construction with timeout=Some(5) must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid/api/v1");
    assert_eq!(c.site_name(), "alps");
  }

  #[test]
  fn new_with_timeout_prepends_http_scheme_when_missing() {
    let c = MantaClient::new_with_timeout("stub.invalid:8080", "alps", None)
      .expect("scheme-less host must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid:8080/api/v1");
  }

  /// When the auth path against an unreachable server fails, the
  /// error chain MUST contain the [`AuthServerUnreachable`] marker.
  /// `common::authentication::get_api_token` downcasts looking for
  /// this to short-circuit instead of falling through to the next
  /// auth source (env-var → file → interactive prompt loop). If
  /// `get_token`/`validate_token` ever stops attaching the marker
  /// on connect/timeout, the CLI silently regresses to "ask for
  /// credentials repeatedly against a dead server".
  #[tokio::test]
  async fn unreachable_server_attaches_auth_marker_to_error_chain() {
    use super::AuthServerUnreachable;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let client = MantaClient::new_with_timeout(
      &format!("http://{addr}"),
      "test-site",
      Some(2),
    )
    .unwrap();

    let err = client
      .get_token("u", "p")
      .await
      .expect_err("auth against closed port must error");
    assert!(
      err.downcast_ref::<AuthServerUnreachable>().is_some(),
      "auth error must be downcastable to AuthServerUnreachable; \
       got: {err:#}",
    );
  }

  /// When the configured manta server URL points at a closed port,
  /// the helper-shaped HTTP call must fail with the meaningful
  /// "cannot reach manta server at <url>" message, NOT a raw reqwest
  /// "error sending request" string. Pinned because the message is
  /// what the operator sees in the terminal when their server is
  /// down or misconfigured.
  #[tokio::test]
  async fn unreachable_server_produces_meaningful_error_message() {
    // Grab a port from the OS, then drop the listener so the
    // address is reachable (route exists) but nothing is bound to
    // it — TCP connect returns ECONNREFUSED. More deterministic
    // than picking a static port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let client = MantaClient::new_with_timeout(
      &format!("http://{addr}"),
      "test-site",
      Some(2),
    )
    .unwrap();

    let result: anyhow::Result<serde_json::Value> =
      client.get_json("test-token", "/health", &[]).await;

    let err = result.expect_err("connect to closed port must error");
    let msg = format!("{err:#}");
    assert!(
      msg.contains("cannot reach manta server"),
      "error must name the unreachability, got: {msg}",
    );
    assert!(
      msg.contains(&format!("http://{addr}")),
      "error must include the configured server URL, got: {msg}",
    );
    assert!(
      msg.contains("manta_server_url"),
      "error must hint at the config knob, got: {msg}",
    );
  }

  /// Behavioural test: when a timeout is set, an outbound call against
  /// a TCP listener that accepts but never responds must error within
  /// roughly the configured window — far below the wall-clock cap.
  #[tokio::test]
  async fn new_with_timeout_some_fires_within_configured_window() {
    use std::time::Duration;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
      if let Ok((_socket, _)) = listener.accept().await {
        tokio::time::sleep(Duration::from_secs(5)).await;
      }
    });

    let client = MantaClient::new_with_timeout(
      &format!("http://{addr}"),
      "test-site",
      Some(1),
    )
    .unwrap();

    let start = std::time::Instant::now();
    let result: anyhow::Result<serde_json::Value> = tokio::time::timeout(
      Duration::from_secs(3),
      client.post_json("test-token", "/health", &serde_json::json!({})),
    )
    .await
    .expect("test cap (3s) exceeded — the inner timeout never fired");
    let elapsed = start.elapsed();

    assert!(result.is_err(), "the call should have errored on timeout");
    assert!(
      elapsed < Duration::from_secs(3),
      "elapsed {elapsed:?} suggests the configured 1s timeout did not fire"
    );

    server.abort();
  }

  /// Back-compat check: a client built without a timeout does NOT
  /// time out at one second.
  #[tokio::test]
  async fn new_without_timeout_does_not_fire_inside_one_second() {
    use std::time::Duration;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
      if let Ok((_socket, _)) = listener.accept().await {
        tokio::time::sleep(Duration::from_secs(5)).await;
      }
    });

    let client =
      MantaClient::new(&format!("http://{addr}"), "test-site").unwrap();

    let result = tokio::time::timeout(
      Duration::from_millis(1500),
      client.post_json::<serde_json::Value>(
        "test-token",
        "/health",
        &serde_json::json!({}),
      ),
    )
    .await;

    assert!(
      result.is_err(),
      "MantaClient::new should not apply a per-request timeout"
    );

    server.abort();
  }
}
