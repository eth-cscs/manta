//! `MantaClient` — the HTTP client struct + shared
//! `get_json`/`post_json`/`put_no_content`/`delete_*` helpers that the
//! per-resource sibling modules dispatch through.

use anyhow::{Context, bail};
use serde::de::DeserializeOwned;

use super::wire::format_request_as_curl;

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

  /// User-facing one-liner used when an outbound request fails before
  /// the server can respond (TCP refused, DNS failure, request timeout).
  /// Names the server URL the CLI tried and hints at the two most
  /// common causes — server not running, or wrong `manta_server_url`.
  pub(super) fn unreachable_server_msg(&self) -> String {
    let server_url = self.base_url.trim_end_matches("/api/v1");
    format!(
      "cannot reach manta server at {server_url}. Is the server \
       running, and is `manta_server_url` in your config correct?"
    )
  }

  /// Map a `reqwest::Error` from `.send().await` into an
  /// `anyhow::Error`. Connect-level and timeout failures get the
  /// human-readable [`Self::unreachable_server_msg`] as the
  /// top-level context; everything else falls back to a generic
  /// `HTTP <METHOD> failed`. The underlying reqwest error is kept
  /// in the cause chain so `manta -v` still surfaces the low-level
  /// detail.
  fn map_send_error(&self, e: reqwest::Error, method: &str) -> anyhow::Error {
    if e.is_connect() || e.is_timeout() {
      anyhow::Error::new(e).context(self.unreachable_server_msg())
    } else {
      anyhow::Error::new(e).context(format!("HTTP {method} failed"))
    }
  }

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
      bail!("Server returned {status}: {}", unwrap_error_body(&body))
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
      bail!("Server returned {status}: {}", unwrap_error_body(&body))
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "GET"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "POST"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "PUT"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "DELETE"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "DELETE"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "DELETE"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "DELETE"))?;
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
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_send_error(e, "DELETE"))?;
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

/// Try to read the server's standard `{"error": "..."}` body and
/// return the inner message. Falls back to the raw body for non-
/// conforming error responses (e.g. axum's default 405 body, an
/// upstream reverse proxy's error page) so we never lose information.
///
/// Keeps the user-facing error short and human — "Server returned 400
/// Bad Request: Can't access HSM group 'compute-2'." instead of
/// "Server returned 400 Bad Request: {\"error\":\"Can't access HSM
/// group 'compute-2'.\"}".
fn unwrap_error_body(body: &str) -> String {
  #[derive(serde::Deserialize)]
  struct ServerErrorBody {
    error: String,
  }
  serde_json::from_str::<ServerErrorBody>(body)
    .map(|e| e.error)
    .unwrap_or_else(|_| body.to_string())
}

#[cfg(test)]
mod tests {
  use super::unwrap_error_body;

  #[test]
  fn unwrap_error_body_extracts_error_field_from_standard_body() {
    let body = r#"{"error":"Can't access HSM group 'compute-2'."}"#;
    assert_eq!(
      unwrap_error_body(body),
      "Can't access HSM group 'compute-2'."
    );
  }

  #[test]
  fn unwrap_error_body_falls_back_to_raw_for_non_json() {
    let body = "Method Not Allowed";
    assert_eq!(unwrap_error_body(body), "Method Not Allowed");
  }

  #[test]
  fn unwrap_error_body_falls_back_to_raw_for_json_without_error_field() {
    let body = r#"{"detail":"something"}"#;
    assert_eq!(unwrap_error_body(body), body);
  }

  #[test]
  fn unwrap_error_body_falls_back_to_raw_for_empty_body() {
    assert_eq!(unwrap_error_body(""), "");
  }
}
