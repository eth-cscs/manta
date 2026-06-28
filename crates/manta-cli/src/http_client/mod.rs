//! Thin HTTP client for forwarding CLI calls to the manta server.
//!
//! # Auto-generated vs. hand-rolled split
//!
//! Most of the API surface — every plain JSON request/response endpoint
//! — comes from the **progenitor-generated** typed client at
//! [`crate::openapi_client::Client`], wrapped in [`MantaClient`]
//! (see `client.rs`). The generated client is regenerated on every
//! build from `crates/manta-cli/openapi.json`. Dispatch handlers
//! reach those endpoints through:
//!
//! ```ignore
//! let client = MantaClient::from_app_ctx(ctx, Some(token))?;
//! let groups = client
//!   .openapi
//!   .get_groups(params.group_name.as_deref(), client.site_name())
//!   .await
//!   .into_anyhow()?;
//! ```
//!
//! The progenitor result is converted via [`OpenApiResultExt::into_anyhow`].
//!
//! Two endpoint families do NOT fit the generated-client shape — they
//! use HTTP upgrade or long-poll streaming protocols that progenitor
//! doesn't model — and live as **hand-rolled** `impl MantaClient`
//! methods on the raw `reqwest::Client`:
//!
//! - WebSocket consoles → `console.rs`
//! - SSE log streaming  → `streaming.rs`
//!
//! When adding a new endpoint, the default path is: annotate the
//! server handler with `#[utoipa::path(...)]`, regenerate the spec,
//! and the generated client picks it up. Hand-rolling here is only
//! the answer when the wire protocol falls outside the generated
//! client's request/response model.

mod client;
mod console;
mod streaming;
mod wire;

pub use client::{
  AuthServerUnreachable, MantaClient, OpenApiResultExt, SiteNotFound,
};
pub(super) use wire::ws_base_url;

#[cfg(test)]
mod tests {
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

  // ---- MantaClient constructors ----

  use super::MantaClient;

  #[test]
  fn new_with_timeout_none_pins_url_and_site() {
    let c =
      MantaClient::new_with_timeout("http://stub.invalid", "alps", None, None)
        .expect("construction with timeout=None must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid/api/v1");
    assert_eq!(c.site_name(), "alps");
  }

  #[test]
  fn new_with_timeout_some_pins_url_and_site() {
    let c = MantaClient::new_with_timeout(
      "http://stub.invalid",
      "alps",
      Some(5),
      None,
    )
    .expect("construction with timeout=Some(5) must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid/api/v1");
    assert_eq!(c.site_name(), "alps");
  }

  #[test]
  fn new_with_timeout_prepends_http_scheme_when_missing() {
    let c =
      MantaClient::new_with_timeout("stub.invalid:8080", "alps", None, None)
        .expect("scheme-less host must succeed");
    assert_eq!(c.base_url(), "http://stub.invalid:8080/api/v1");
  }
}
