//! `MantaClient` — thin wrapper around the progenitor-generated
//! `openapi_client::Client`, plus a `reqwest::Client` + token for the
//! WebSocket / SSE paths that bypass the generated client.
//!
//! Dispatch handlers reach the API through `client.openapi.<method>(...)`
//! and convert the progenitor result with [`OpenApiResultExt::into_anyhow`].
//! Console / log-streaming endpoints sit on `impl MantaClient` (see
//! `console.rs`, `streaming.rs`) and use the raw `reqwest::Client`
//! (with default `Authorization` header set in the constructor).
//!
//! Bearer auth is wired once in the constructor via
//! `reqwest::ClientBuilder::default_headers`; the only call sites that
//! pass `None` are `common::authentication` (which is how we *obtain*
//! or *validate* the token in the first place).

use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};

use super::wire::format_request_as_curl;

/// Marker error attached as anyhow context whenever an `/auth/*`
/// HTTP call fails at the TCP/timeout layer (i.e., the manta server
/// — and therefore the auth path through it — is unreachable, not
/// "wrong credentials"). Lets [`crate::common::authentication`] tell
/// the two cases apart so an unreachable server short-circuits
/// instead of triggering the re-prompt loop.
#[derive(Debug)]
pub struct AuthServerUnreachable {
  /// The base manta server URL (without `/api/v1`) that was tried.
  /// Surfaced in the error message and recoverable by the loop via
  /// `downcast_ref` if a caller needs to log it separately.
  pub url: String,
}

impl std::fmt::Display for AuthServerUnreachable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "cannot reach manta server at {} for authentication. \
       Is the server running, and is `manta_server_url` in your \
       config correct?",
      self.url,
    )
  }
}

impl std::error::Error for AuthServerUnreachable {}

/// Convert a `Result<ResponseValue<T>, Error<E>>` from the
/// progenitor-generated client into an `anyhow::Result<T>`.
///
/// Implementation: on `Err`, format the progenitor error via
/// `Display` and wrap with `anyhow::anyhow!(...)`. Progenitor's
/// `Display` impl on `Error<T>` includes the body for `ErrorResponse`
/// cases, so the inner server-supplied detail is preserved.
pub trait OpenApiResultExt<T> {
  fn into_anyhow(self) -> anyhow::Result<T>;
}

impl<T, E> OpenApiResultExt<T>
  for Result<progenitor_client::ResponseValue<T>, progenitor_client::Error<E>>
where
  E: std::fmt::Debug,
{
  fn into_anyhow(self) -> anyhow::Result<T> {
    match self {
      Ok(rv) => Ok(rv.into_inner()),
      Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
  }
}

/// HTTP client that forwards CLI requests to a manta server.
///
/// Two transports inside:
/// - `openapi` — progenitor-generated typed client. Used by every
///   dispatch handler for the API surface declared in `openapi.json`.
/// - `raw` — plain `reqwest::Client` (same bearer-auth default header
///   as `openapi`). Used by the WebSocket consoles and SSE log stream
///   that aren't part of the generated client surface.
#[derive(Debug)]
pub struct MantaClient {
  /// Generated typed API client. Dispatch handlers call
  /// `client.openapi.<method>(...).await.into_anyhow()?`.
  pub openapi: crate::openapi_client::Client,
  /// `X-Manta-Site` header value the server uses to pick the backend
  /// config. Threaded explicitly to every `openapi.*` call.
  pub site_name: String,
  /// Plain `reqwest::Client` for the WS / SSE paths. Same bearer-auth
  /// default header as `openapi` (when a token was passed in).
  pub raw: reqwest::Client,
  /// Bearer token kept around for the WS path (which builds its own
  /// `Authorization` header on the tungstenite request).
  pub token: Option<String>,
  /// Base URL `<http(s)://host:port>/api/v1` used by the WS / SSE
  /// paths to build `wss://…` and `…/sessions/{name}/logs` URLs.
  pub base_url: String,
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
    Self::new_with_timeout(server_url, site_name, None, None)
  }

  /// Build a client from an `AppContext`, honouring its
  /// `request_timeout_secs` (loaded from `cli.toml`).
  ///
  /// `token` is wired as a default `Authorization: Bearer <t>` header
  /// on both the `openapi` client and the `raw` `reqwest::Client`
  /// when `Some`. Pass `None` only for the auth path that *obtains*
  /// the token (`common::authentication`).
  pub fn from_app_ctx(
    ctx: &crate::common::app_context::AppContext<'_>,
    token: Option<&str>,
  ) -> anyhow::Result<Self> {
    Self::new_with_timeout(
      ctx.manta_server_url,
      ctx.site_name,
      ctx.request_timeout_secs,
      token,
    )
  }

  /// Build a client with an explicit optional per-request timeout
  /// and optional bearer token.
  ///
  /// When `token` is `Some(t)`, the underlying `reqwest::Client`s get
  /// a default `Authorization: Bearer <t>` header so every call —
  /// generated or raw — sends the auth header automatically.
  ///
  /// `None` for `timeout_secs` keeps reqwest's default (no timeout);
  /// `Some(secs)` configures the inner `reqwest::Client` with
  /// `.timeout(Duration::from_secs(secs))`. URL scheme normalisation
  /// matches [`MantaClient::new`].
  pub fn new_with_timeout(
    server_url: &str,
    site_name: &str,
    timeout_secs: Option<u64>,
    token: Option<&str>,
  ) -> anyhow::Result<Self> {
    let normalized = if server_url.starts_with("http://")
      || server_url.starts_with("https://")
    {
      server_url.to_owned()
    } else {
      format!("http://{server_url}")
    };
    let base_url = format!("{}/api/v1", normalized.trim_end_matches('/'));

    let mut default_headers = HeaderMap::new();
    if let Some(t) = token {
      let mut bearer = HeaderValue::from_str(&format!("Bearer {t}"))
        .context("token contained non-ASCII characters; cannot build Authorization header")?;
      bearer.set_sensitive(true);
      default_headers.insert(reqwest::header::AUTHORIZATION, bearer);
    }

    let mut builder = reqwest::Client::builder().default_headers(default_headers);
    if let Some(secs) = timeout_secs {
      builder = builder.timeout(std::time::Duration::from_secs(secs));
    }
    let raw = builder.build().context("Failed to build HTTP client")?;

    let openapi =
      crate::openapi_client::Client::new_with_client(&base_url, raw.clone());

    Ok(Self {
      openapi,
      site_name: site_name.to_owned(),
      raw,
      token: token.map(str::to_owned),
      base_url,
    })
  }

  /// Site name accessor used at every `openapi.<method>(args, client.site_name())` call site.
  pub fn site_name(&self) -> &str {
    &self.site_name
  }

  /// Base URL `<scheme>://<host>/api/v1` — used by the WebSocket /
  /// SSE paths to derive their own URLs.
  pub fn base_url(&self) -> &str {
    &self.base_url
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
}

/// Try to read the server's standard `{"error": "..."}` body and
/// return the inner message. Falls back to the raw body for non-
/// conforming error responses (e.g. axum's default 405 body, an
/// upstream reverse proxy's error page) so we never lose information.
///
/// Still used by the SSE log stream path which builds its request
/// by hand and renders the server's error body directly to the user.
pub(super) fn unwrap_error_body(body: &str) -> String {
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
