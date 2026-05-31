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

use anyhow::{Context, bail};
use serde::de::DeserializeOwned;

mod auth;
mod boot_parameters;
mod clusters;
mod configurations;
mod console;
mod ephemeral_env;
mod groups;
mod hardware;
mod hw_cluster;
mod images;
mod kernel_parameters;
mod migrate;
mod nodes;
mod power;
mod redfish_endpoints;
mod sat_file;
mod sessions;
mod templates;

/// HTTP client that forwards CLI requests to a manta server.
#[derive(Debug)]
pub struct MantaClient {
  client: reqwest::Client,
  base_url: String,
  site_name: String,
}

impl MantaClient {
  /// Build a client pointing at `server_url` for the given `site_name`.
  /// No per-request HTTP timeout is set; reqwest's default applies
  /// (which is "no timeout"). For a configurable timeout, use
  /// [`MantaClient::new_with_timeout`].
  ///
  /// If `server_url` has no scheme, `http://` is prepended. This lets users
  /// write `manta_server_url = "localhost:8080"` in their config without
  /// triggering a "URL scheme is not allowed" error from reqwest.
  pub fn new(server_url: &str, site_name: &str) -> anyhow::Result<Self> {
    Self::new_with_timeout(server_url, site_name, None)
  }

  /// Build a client with an explicit optional per-request timeout.
  /// `None` keeps reqwest's default (no timeout); `Some(secs)`
  /// configures the inner `reqwest::Client` with
  /// `.timeout(Duration::from_secs(secs))`.
  pub fn new_with_timeout(
    server_url: &str,
    site_name: &str,
    timeout_secs: Option<u64>,
  ) -> anyhow::Result<Self> {
    let normalized = if server_url.starts_with("http://")
      || server_url.starts_with("https://")
    {
      server_url.to_owned()
    } else {
      format!("http://{server_url}")
    };

    let mut builder = reqwest::Client::builder();
    if let Some(secs) = timeout_secs {
      builder = builder.timeout(std::time::Duration::from_secs(secs));
    }
    let client = builder.build().context("Failed to build HTTP client")?;
    Ok(Self {
      client,
      base_url: format!("{}/api/v1", normalized.trim_end_matches('/')),
      site_name: site_name.to_owned(),
    })
  }

  /// Emit a `curl` equivalent of `builder` at DEBUG level so an operator
  /// can replay the request from their shell. Skipped entirely when
  /// DEBUG is filtered out, so the clone/build/serialize cost is only
  /// paid when the log will actually be emitted.
  ///
  /// Secrets-safe: the `Authorization` header value is masked, and
  /// `password` / `token` fields anywhere in a JSON body are replaced
  /// with `<REDACTED>` before formatting.
  pub(super) fn log_request_as_curl(builder: &reqwest::RequestBuilder) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
      return;
    }
    let Some(cloned) = builder.try_clone() else {
      return;
    };
    let Ok(req) = cloned.build() else {
      return;
    };
    tracing::debug!(
      "curl equivalent (secrets replaced with <REDACTED>):\n{}",
      format_request_as_curl(&req)
    );
  }

  // ── shared helpers (visible to sub-modules) ───────────────────────────────
  //
  // These are `pub(super)` so the resource sub-modules can call them. They
  // are the only places that touch `reqwest` directly; sub-module methods
  // build a URL fragment + query/body and delegate here.

  pub(super) async fn parse_json<T: DeserializeOwned>(
    resp: reqwest::Response,
  ) -> anyhow::Result<T> {
    if resp.status().is_success() {
      resp
        .json::<T>()
        .await
        .context("Failed to parse response JSON")
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {status}: {body}")
    }
  }

  pub(super) async fn parse_no_content(
    resp: reqwest::Response,
  ) -> anyhow::Result<()> {
    if resp.status().is_success() {
      Ok(())
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {status}: {body}")
    }
  }

  pub(super) async fn get_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .get(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP GET failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn post_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .post(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP POST failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn put_no_content(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .put(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP PUT failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content(
    &self,
    token: &str,
    path: &str,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content_with_query(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content_with_body(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_json_with_body<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn delete_json_with_query<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let builder = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query);
    Self::log_request_as_curl(&builder);
    let resp = builder.send().await.context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  // Accessors used by sub-modules that build URLs / set headers directly
  // (SSE streaming, WebSocket consoles).
  pub(super) fn http_client(&self) -> &reqwest::Client {
    &self.client
  }
  pub(super) fn base_url(&self) -> &str {
    &self.base_url
  }
  pub(super) fn site_name(&self) -> &str {
    &self.site_name
  }
}

/// Chainable builder for the `&[(&str, String)]` query-pairs slice
/// that `MantaClient::get_json` expects. Each `.opt()` / `.vec()` /
/// `.flag()` / `.pair()` call mirrors one of the patterns the older
/// hand-written query blocks used; absent values are skipped.
#[derive(Default)]
pub(super) struct QueryBuilder {
  pairs: Vec<(&'static str, String)>,
}

impl QueryBuilder {
  pub(super) fn new() -> Self {
    Self::default()
  }

  /// Push `(name, value.clone())` only when `value` is `Some`.
  pub(super) fn opt(
    mut self,
    name: &'static str,
    value: &Option<String>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.clone()));
    }
    self
  }

  /// Push `(name, value.to_string())` only when `value` is `Some`.
  /// For numeric `Option<T>` where `T: ToString`.
  pub(super) fn opt_display<T: ToString>(
    mut self,
    name: &'static str,
    value: &Option<T>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.to_string()));
    }
    self
  }

  /// Push `(name, items.join(","))` only when `items` is non-empty.
  pub(super) fn vec(mut self, name: &'static str, items: &[String]) -> Self {
    if !items.is_empty() {
      self.pairs.push((name, items.join(",")));
    }
    self
  }

  /// Push `(name, "true")` only when `value` is `true`.
  pub(super) fn flag(mut self, name: &'static str, value: bool) -> Self {
    if value {
      self.pairs.push((name, "true".to_string()));
    }
    self
  }

  /// Push `(name, value)` unconditionally.
  pub(super) fn pair(mut self, name: &'static str, value: String) -> Self {
    self.pairs.push((name, value));
    self
  }

  /// Consume into the slice-shaped form `get_json` accepts.
  pub(super) fn build(self) -> Vec<(&'static str, String)> {
    self.pairs
  }
}

/// Render `req` as a copy-pasteable `curl` invocation. Used by
/// [`MantaClient::log_request_as_curl`]; the secrets-redaction policy
/// lives here so it's consistent across every call site.
fn format_request_as_curl(req: &reqwest::Request) -> String {
  let mut out = format!("  curl -k -X {} '{}'", req.method(), req.url());
  for (name, value) in req.headers() {
    let raw = value.to_str().unwrap_or("<binary>");
    let rendered = if name == reqwest::header::AUTHORIZATION {
      if raw.starts_with("Bearer ") {
        "Bearer <REDACTED>".to_string()
      } else {
        "<REDACTED>".to_string()
      }
    } else {
      raw.to_string()
    };
    out.push_str(&format!(" \\\n    -H '{name}: {rendered}'"));
  }
  if let Some(body_bytes) = req.body().and_then(reqwest::Body::as_bytes) {
    let body_str = std::str::from_utf8(body_bytes).unwrap_or("<binary>");
    let redacted = redact_json_secrets(body_str);
    out.push_str(&format!(" \\\n    --data-raw '{redacted}'"));
  }
  out
}

/// Walk `body` as JSON, replacing any `password` or `token` field value
/// with `<REDACTED>`. Falls back to the original string when the body
/// isn't parseable as JSON — non-JSON bodies are rare on this client
/// and never carry credentials.
fn redact_json_secrets(body: &str) -> String {
  let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body) else {
    return body.to_string();
  };
  redact_value(&mut value);
  serde_json::to_string(&value).unwrap_or_else(|_| body.to_string())
}

fn redact_value(v: &mut serde_json::Value) {
  match v {
    serde_json::Value::Object(map) => {
      for (k, val) in map.iter_mut() {
        if matches!(k.as_str(), "password" | "token") {
          *val = serde_json::Value::String("<REDACTED>".to_string());
        } else {
          redact_value(val);
        }
      }
    }
    serde_json::Value::Array(arr) => {
      for item in arr {
        redact_value(item);
      }
    }
    _ => {}
  }
}

/// Convert an `http://` or `https://` base URL to the corresponding `ws://` / `wss://` URL.
pub(super) fn ws_base_url(http_url: &str) -> String {
  if let Some(rest) = http_url.strip_prefix("https://") {
    format!("wss://{rest}")
  } else if let Some(rest) = http_url.strip_prefix("http://") {
    format!("ws://{rest}")
  } else {
    http_url.to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn query_builder_opt_display_skips_none_includes_some() {
    let q = QueryBuilder::new()
      .opt_display::<u32>("limit", &None)
      .opt_display("offset", &Some(42u32))
      .build();
    assert_eq!(q, vec![("offset", "42".to_string())]);
  }

  #[test]
  fn query_builder_vec_skips_empty_joins_with_comma() {
    let empty: &[String] = &[];
    let q1 = QueryBuilder::new().vec("xnames", empty).build();
    assert!(q1.is_empty(), "empty vec must produce no pair");

    let items = vec!["x1".to_string(), "x2".to_string(), "x3".to_string()];
    let q2 = QueryBuilder::new().vec("xnames", &items).build();
    assert_eq!(q2, vec![("xnames", "x1,x2,x3".to_string())]);
  }

  #[test]
  fn query_builder_flag_only_emits_when_true() {
    // false skips entirely — the server's bool extractor distinguishes
    // "missing" from "false", so emitting "false" when not set would
    // change semantics for some endpoints.
    let q1 = QueryBuilder::new().flag("verbose", false).build();
    assert!(q1.is_empty());

    let q2 = QueryBuilder::new().flag("verbose", true).build();
    assert_eq!(q2, vec![("verbose", "true".to_string())]);
  }

  #[test]
  fn query_builder_pair_always_emits() {
    let q = QueryBuilder::new()
      .pair("site", "alps".to_string())
      .pair("kind", "node".to_string())
      .build();
    assert_eq!(
      q,
      vec![("site", "alps".to_string()), ("kind", "node".to_string()),]
    );
  }

  #[test]
  fn query_builder_preserves_insertion_order() {
    // Order matters because some servers/proxies are sensitive to it;
    // pin the behaviour so a future Map-based refactor wouldn't silently
    // change request shape.
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
    // Realistic usage: a get-nodes command builds a query with optional
    // filters, a flag, and a multi-value xnames list.
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

  #[test]
  fn new_with_timeout_none_builds_successfully() {
    let c = MantaClient::new_with_timeout("http://stub.invalid", "alps", None);
    assert!(c.is_ok());
  }

  #[test]
  fn new_with_timeout_some_builds_successfully() {
    let c = MantaClient::new_with_timeout("http://stub.invalid", "alps", Some(5));
    assert!(c.is_ok());
  }

  /// Behavioural test: when a timeout is set, an outbound call against
  /// a TCP listener that accepts but never responds must error within
  /// roughly the configured window — far below the wall-clock cap.
  /// Confirms `.timeout(...)` is actually wired into the reqwest
  /// client.
  #[tokio::test]
  async fn new_with_timeout_some_fires_within_configured_window() {
    use std::time::Duration;

    // Accept-but-hang listener: takes the connection, then sleeps so
    // the HTTP response never arrives.
    let listener =
      tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
      // Accept exactly one connection and hold it open for longer than
      // the test cap so the timeout fires before we drop the socket.
      if let Ok((_socket, _)) = listener.accept().await {
        tokio::time::sleep(Duration::from_secs(5)).await;
      }
    });

    let client = MantaClient::new_with_timeout(
      &format!("http://{addr}"),
      "test-site",
      Some(1), // 1 second
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

  /// Back-compat check: a client built without a timeout (the default
  /// for every non-power command today) does NOT time out at one
  /// second — proves the new code path didn't silently leak a default
  /// timeout into `MantaClient::new`.
  #[tokio::test]
  async fn new_without_timeout_does_not_fire_inside_one_second() {
    use std::time::Duration;

    let listener =
      tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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

    // The outer tokio::time::timeout MUST fire (Err) — meaning the
    // inner reqwest call was still pending at 1.5s, which is what we
    // want: no client-side timeout applied.
    assert!(
      result.is_err(),
      "MantaClient::new should not apply a per-request timeout"
    );

    server.abort();
  }
}
