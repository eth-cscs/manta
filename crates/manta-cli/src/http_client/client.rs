//! `MantaClient` — thin wrapper around the **auto-generated**
//! `openapi_client::Client` (see `crate::openapi_client`), plus a
//! `reqwest::Client` + token for the **hand-rolled** WebSocket /
//! SSE paths that bypass the generated client.
//!
//! Dispatch handlers reach the auto-generated API surface through
//! `client.openapi.<method>(...)` and convert the progenitor result
//! with [`OpenApiResultExt::into_anyhow`]. Hand-rolled console /
//! log-streaming endpoints sit on `impl MantaClient` in `console.rs`
//! and `streaming.rs` respectively, and use the raw `reqwest::Client`
//! (with default `Authorization` header set in the constructor).
//!
//! Bearer auth is wired once in the constructor via
//! `reqwest::ClientBuilder::default_headers`; the only call sites that
//! pass `None` are `common::authentication` (which is how we *obtain*
//! or *validate* the token in the first place).

use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};

use super::wire::format_request_as_curl;

/// Default per-request timeout applied to one-shot REST calls through
/// the progenitor-generated `openapi` client when `cli.toml` does not
/// set `request_timeout_secs`. Five minutes matches the manta-server
/// default global request timeout, so the CLI gives up at roughly the
/// same point the server would have anyway. Streams (SSE log tail,
/// WebSocket consoles) keep the original `None == unlimited` semantics
/// — see [`MantaClient::new_with_timeout`].
pub const DEFAULT_API_TIMEOUT_SECS: u64 = 300;

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

/// Marker error attached as anyhow context whenever an `/auth/*` HTTP
/// call returns `404 Not Found` — the manta server is reachable but the
/// `site` in your config isn't one it serves. Lets
/// [`crate::common::authentication`] short-circuit the
/// env → file → interactive-prompt cascade instead of asking for
/// credentials that can never succeed against a site that doesn't exist.
#[derive(Debug)]
pub struct SiteNotFound {
  /// The `X-Manta-Site` value the server rejected. Surfaced in the
  /// error message and recoverable via `downcast_ref`.
  pub site: String,
}

impl std::fmt::Display for SiteNotFound {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "site '{}' is not configured on the manta server. \
       Check the `site` value in your `cli.toml`.",
      self.site,
    )
  }
}

impl std::error::Error for SiteNotFound {}

/// Convert a `Result<ResponseValue<T>, Error<E>>` from the
/// progenitor-generated client into an `anyhow::Result<T>`.
///
/// Every dispatch handler in `crate::dispatch::*` chains this onto
/// its `client.openapi.<method>(...).await` call so that the
/// progenitor envelope is normalised into a uniform anyhow error
/// before bubbling up.
///
/// Implementation: on `Err`, format the progenitor error in a
/// user-friendly way:
///
/// - `ErrorResponse(rv)`: the server returned a typed error body. We
///   serialise it to JSON, pull the `error` string out, and surface
///   `HTTP <status>: <message>`. This avoids dumping the full
///   `Error Response: status=…; headers={…}; value: ErrorResponse {…}`
///   envelope that progenitor's `Display` impl produces.
/// - All other variants: format via `Display` (transport errors,
///   payload-decode errors, etc.) — the inner detail is still useful.
pub trait OpenApiResultExt<T> {
  /// Convert the progenitor result into `anyhow::Result<T>`. See the
  /// trait doc for the error-shaping rules.
  ///
  /// # Errors
  ///
  /// Returns `Err` whenever the wrapped `Result` is `Err`. The
  /// concrete message depends on the progenitor variant — see the
  /// trait-level doc.
  fn into_anyhow(self) -> anyhow::Result<T>;
}

impl<T, E> OpenApiResultExt<T>
  for Result<progenitor_client::ResponseValue<T>, progenitor_client::Error<E>>
where
  E: std::fmt::Debug + serde::Serialize,
{
  fn into_anyhow(self) -> anyhow::Result<T> {
    match self {
      Ok(rv) => Ok(rv.into_inner()),
      Err(progenitor_client::Error::ErrorResponse(rv)) => {
        let status = rv.status();
        let inner = rv.into_inner();
        let raw_msg = serde_json::to_value(&inner)
          .ok()
          .and_then(|v| {
            v.get("error").and_then(|e| e.as_str()).map(String::from)
          })
          .unwrap_or_else(|| format!("{inner:?}"));
        let msg = categorise_server_error(status.as_u16(), &raw_msg);
        Err(anyhow::anyhow!("HTTP {}: {msg}", status.as_u16()))
      }
      Err(progenitor_client::Error::CommunicationError(rqe)) => {
        Err(anyhow::anyhow!("{}", categorise_transport_error(&rqe)))
      }
      Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
  }
}

/// Re-shape a `reqwest::Error` from the CLI-to-manta-server transport
/// hop into a message that names *which* timeout fired, when it fired.
/// The CLI itself owns two timeouts on this hop (`connect_timeout` and
/// `request_timeout_secs` in `cli.toml`), and reqwest exposes
/// `is_connect()` / `is_timeout()` to distinguish.
fn categorise_transport_error(rqe: &reqwest::Error) -> String {
  if rqe.is_timeout() {
    if rqe.is_connect() {
      format!(
        "CLI -> manta-server connect timed out. \
         The CLI gave up before the TCP/TLS handshake completed. \
         Likely causes: manta-server unreachable, wrong `manta_server_url` \
         in `cli.toml`, or a firewall dropping the SYN. \
         Underlying: {rqe}"
      )
    } else {
      format!(
        "CLI -> manta-server request timed out. \
         The CLI gave up before manta-server sent response headers. \
         This is the CLI-side `request_timeout_secs` in `cli.toml` \
         (default 300 s). manta-server may still be working on the \
         request — bump that value if you're hitting it on a heavy \
         call against a busy site. Underlying: {rqe}"
      )
    }
  } else if rqe.is_connect() {
    format!(
      "CLI could not connect to manta-server. \
       Check `manta_server_url` in `cli.toml` and confirm the server \
       is reachable from this host. Underlying: {rqe}"
    )
  } else {
    format!("CLI transport error talking to manta-server: {rqe}")
  }
}

/// Re-shape an `HTTP <status>: <body>` error from manta-server into a
/// message that names *which* timeout fired when the body matches a
/// known timeout shape. Only the timeout-related cases are rewritten;
/// everything else passes through as-is.
fn categorise_server_error(status: u16, body: &str) -> String {
  if status == 408 {
    return format!(
      "manta-server per-route request timeout fired. \
       The handler took longer than `request_timeout_secs` in \
       `server.toml` (default 600 s). The upstream call may still \
       be running on the server. Original body: {body}"
    );
  }
  // The CSM HTTP client surfaces timeouts via `Error::NetError(reqwest::Error)`,
  // which Display-formats with the literal "operation timed out" string.
  // When the server's body carries that shape (after to_handler_error's
  // own rewrite), flag it as a CSM-side timeout so the user knows the
  // hop that timed out wasn't CLI->manta-server, it was manta-server->CSM.
  if body.contains("operation timed out")
    || body.contains("Connect timed out")
    || body.contains("manta-server -> CSM")
  {
    return format!(
      "manta-server's outbound call to CSM timed out (csm-rs \
       reqwest timeout). This is not the CLI or manta-server's own \
       timeout — CSM itself did not respond. Original body: {body}"
    );
  }
  body.to_string()
}

#[cfg(test)]
mod into_anyhow_tests {
  use super::*;

  #[test]
  fn categorise_server_408_explains_route_timeout() {
    let msg = categorise_server_error(408, "Request Timeout");
    assert!(msg.contains("per-route request timeout"));
    assert!(msg.contains("server.toml"));
    // Original body is still surfaced for debug.
    assert!(msg.contains("Request Timeout"));
  }

  #[test]
  fn categorise_server_500_with_operation_timed_out_explains_csm_hop() {
    let msg = categorise_server_error(
      500,
      "ERROR - http client: error sending request for url (...): operation timed out",
    );
    assert!(msg.contains("manta-server's outbound call to CSM"));
    assert!(msg.contains("csm-rs"));
  }

  #[test]
  fn categorise_server_500_passes_unknown_bodies_through() {
    let body = "Internal error: something else";
    assert_eq!(categorise_server_error(500, body), body);
  }

  #[test]
  fn categorise_server_404_passes_through() {
    let body = "Not found: image abcd-1234";
    assert_eq!(categorise_server_error(404, body), body);
  }
}

/// HTTP client that forwards CLI requests to a manta server.
///
/// Wraps three things:
/// - the progenitor-generated `crate::openapi_client::Client` (the
///   `openapi` field) for every endpoint declared in `openapi.json`;
/// - a `reqwest::Client` (`raw`) sharing the same bearer-auth default
///   header, used by the hand-rolled WebSocket / SSE paths;
/// - the base URL (`<scheme>://host:port/api/v1`) and the
///   `X-Manta-Site` header value the server uses to pick the active
///   backend config.
///
/// Two transports inside:
/// - `openapi` — progenitor-generated typed client. Used by every
///   dispatch handler for the API surface declared in `openapi.json`.
/// - `raw` — plain `reqwest::Client` (same bearer-auth default header
///   as `openapi`). Used by the WebSocket consoles and SSE log stream
///   that aren't part of the generated client surface.
///
/// # Example
///
/// ```ignore
/// use crate::common::authentication::get_api_token;
/// use crate::http_client::{MantaClient, OpenApiResultExt};
///
/// let token = get_api_token(ctx).await?;
/// let client = MantaClient::from_app_ctx(ctx, Some(&token))?;
/// let groups = client
///   .openapi
///   .get_groups(None, client.site_name())
///   .await
///   .into_anyhow()?;
/// ```
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
  /// One-shot REST calls get a 5-minute timeout
  /// ([`DEFAULT_API_TIMEOUT_SECS`]); streams (SSE / WebSockets) get no
  /// timeout. To override either, use
  /// [`MantaClient::new_with_timeout`].
  ///
  /// If `server_url` has no scheme, `http://` is prepended. This lets users
  /// write `manta_server_url = "localhost:8080"` in their config without
  /// triggering a "URL scheme is not allowed" error from reqwest.
  ///
  /// # Errors
  ///
  /// Propagates failures from [`MantaClient::new_with_timeout`].
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
  ///
  /// # Errors
  ///
  /// - `ctx.require_site()` fails when no site is set.
  /// - `token` contains non-ASCII characters (rejected by reqwest's
  ///   `HeaderValue` parser).
  /// - Either internal `reqwest::Client::build` fails.
  pub fn from_app_ctx(
    ctx: &crate::common::app_context::AppContext<'_>,
    token: Option<&str>,
  ) -> anyhow::Result<Self> {
    Self::new_with_timeout(
      ctx.manta_server_url,
      ctx.require_site()?,
      ctx.request_timeout_secs,
      token,
    )
  }

  /// Build a client with an explicit optional per-request timeout
  /// and optional bearer token.
  ///
  /// When `token` is `Some(t)`, both underlying `reqwest::Client`s get
  /// a default `Authorization: Bearer <t>` header so every call —
  /// generated or raw — sends the auth header automatically.
  ///
  /// Two separate `reqwest::Client`s are built so streams and one-shot
  /// API calls can have different timeout policies:
  ///
  /// - `openapi` (one-shot REST calls via the progenitor-generated
  ///   client): `timeout_secs.unwrap_or(DEFAULT_API_TIMEOUT_SECS)`. The
  ///   default of 300 s caps stuck calls without making operators
  ///   configure a value; setting `request_timeout_secs` in `cli.toml`
  ///   overrides it.
  /// - `raw` (SSE log streaming + WebSocket consoles): respects the
  ///   `timeout_secs` value verbatim — `None` means no timeout, so a
  ///   long-running CFS image build's log stream stays open until the
  ///   session ends or the underlying connection drops. Setting
  ///   `request_timeout_secs` applies here too, which will truncate
  ///   long streams; pick a value larger than your worst-case session.
  ///
  /// URL scheme normalisation matches [`MantaClient::new`].
  ///
  /// # Errors
  ///
  /// - `token` is not a valid HTTP header value (non-ASCII bytes).
  /// - Either internal `reqwest::Client::build` fails (TLS init,
  ///   resolver init, etc.).
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

    // Streams + WebSockets keep the original Option<u64> semantics
    // (None = no timeout).
    let mut raw_builder =
      reqwest::Client::builder().default_headers(default_headers.clone());
    if let Some(secs) = timeout_secs {
      raw_builder = raw_builder.timeout(std::time::Duration::from_secs(secs));
    }
    let raw = raw_builder.build().context("Failed to build HTTP client")?;

    // One-shot API calls default to 5 minutes when nothing is
    // configured. An explicit cli.toml override wins.
    let api_timeout_secs = timeout_secs.unwrap_or(DEFAULT_API_TIMEOUT_SECS);
    let openapi_inner = reqwest::Client::builder()
      .default_headers(default_headers)
      .timeout(std::time::Duration::from_secs(api_timeout_secs))
      .build()
      .context("Failed to build OpenAPI HTTP client")?;
    let openapi =
      crate::openapi_client::Client::new_with_client(&base_url, openapi_inner);

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
