//! `MantaClient` — thin wrapper around the **auto-generated**
//! `openapi_client::Client` (see `crate::openapi_client`), plus a
//! `reqwest::Client` for the **hand-rolled** WebSocket / SSE paths
//! that bypass the generated client.
//!
//! Dispatch handlers reach the auto-generated API surface through
//! `client.openapi.<method>(...)` and convert the progenitor result
//! with [`OpenApiResultExt::into_anyhow`]. Hand-rolled console /
//! log-streaming endpoints sit on `impl MantaClient` in `console.rs`
//! and `streaming.rs` respectively, and use the raw `reqwest::Client`.
//!
//! Every request carries a new bearer token from the client's
//! [`TokenProvider`]: the generated client gets it in its pre-request
//! hook ([`set_current_token`]), the raw paths from
//! [`MantaClient::bearer_token`].

use std::sync::Arc;

use anyhow::Context;
use reqwest::header::HeaderValue;

use super::wire::format_request_as_curl;
use crate::common::authentication::TokenProvider;

/// Default per-request timeout applied to one-shot REST calls through
/// the progenitor-generated `openapi` client when `cli.toml` does not
/// set `request_timeout_secs`. Five minutes matches the manta-server
/// default global request timeout, so the CLI gives up at roughly the
/// same point the server would have anyway. Streams (SSE log tail,
/// WebSocket consoles) keep the original `None == unlimited` semantics
/// — see [`MantaClient::new_with_timeout`].
pub const DEFAULT_API_TIMEOUT_SECS: u64 = 300;

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
/// - `ErrorResponse(rv)`: the server returned a status declared in the
///   OpenAPI spec with a typed body. We serialise it to JSON, pull the
///   `error` string out, and surface `HTTP <status>: <message>`.
/// - `UnexpectedResponse(resp)`: the server returned a status the
///   OpenAPI spec did NOT declare for this operation. The body hasn't
///   been read yet — we consume it here, try to parse `{"error": ...}`
///   out of it, and fall back to the raw body text. This branch is why
///   the method is async: `reqwest::Response::text()` is async and no
///   sync alternative reads the body.
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
  #[allow(async_fn_in_trait)]
  async fn into_anyhow(self) -> anyhow::Result<T>;

  /// [`Self::into_anyhow`] for a call sent with
  /// [`MantaClient::site_hint`]: on success, also adopt the site the
  /// server resolved (its `X-Manta-Site` response header) into
  /// `client`, so the command's follow-up calls target the same site.
  #[allow(async_fn_in_trait)]
  async fn into_anyhow_pinning_site(
    self,
    client: &MantaClient,
  ) -> anyhow::Result<T>;
}

impl<T, E> OpenApiResultExt<T>
  for Result<progenitor_client::ResponseValue<T>, progenitor_client::Error<E>>
where
  E: std::fmt::Debug + serde::Serialize,
{
  async fn into_anyhow_pinning_site(
    self,
    client: &MantaClient,
  ) -> anyhow::Result<T> {
    if let Ok(rv) = &self {
      client.pin_site(rv.headers());
    }
    self.into_anyhow().await
  }

  async fn into_anyhow(self) -> anyhow::Result<T> {
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
      Err(progenitor_client::Error::UnexpectedResponse(resp)) => {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let raw_msg = extract_error_body(&body, status.as_u16());
        let msg = categorise_server_error(status.as_u16(), &raw_msg);
        Err(anyhow::anyhow!("HTTP {}: {msg}", status.as_u16()))
      }
      Err(progenitor_client::Error::CommunicationError(rqe)) => {
        Err(anyhow::anyhow!("{}", categorise_transport_error(&rqe)))
      }
      // The pre-request hook could not get a token; its message says why.
      Err(progenitor_client::Error::Custom(msg)) => Err(anyhow::anyhow!(msg)),
      Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
  }
}

/// The message for a `401`. manta-server exchanges the CSCS token on
/// every request at the site's identity provider (Keycloak on CSM sites,
/// Hydra on OpenCHAMI); a `401` means that provider rejected it. The
/// token was a new one, so it was neither expired nor reused: the cause
/// is who it belongs to, or the site's trust in the CSCS Keycloak.
fn token_rejected(body: &str, service_account: bool) -> String {
  let advice = if service_account {
    "Check that the service account in `MANTA_SA_CLIENT_ID` is allowed on \
     this site"
  } else {
    "Check that your account is allowed on this site; `manta config unset \
     auth` forces a new login, e.g. as another user"
  };
  format!(
    "this site's identity provider rejected the CSCS token. {advice}, or \
     ask the manta-server operators whether the site accepts CSCS tokens. \
     Original body: {body}"
  )
}

/// Pull the human-readable error message out of a raw response body.
/// Recognises the `{"error":"..."}` shape produced by
/// `manta-server`'s `ErrorResponse`; otherwise returns the body
/// verbatim (or a placeholder for empty bodies).
pub(super) fn extract_error_body(body: &str, status: u16) -> String {
  serde_json::from_str::<serde_json::Value>(body)
    .ok()
    .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
    .unwrap_or_else(|| {
      if body.is_empty() {
        format!("<empty response body, status {status}>")
      } else {
        body.to_string()
      }
    })
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
/// message that says what went wrong: a rejected token (401), or
/// *which* timeout fired when the body matches a known timeout shape.
/// Everything else passes through as-is.
fn categorise_server_error(status: u16, body: &str) -> String {
  if status == 401 {
    let service_account = matches!(
      crate::common::authentication::service_account_from_env(),
      Ok(Some(_))
    );
    return token_rejected(body, service_account);
  }
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
  fn a_401_names_the_identity_provider_and_keeps_the_body() {
    let msg = token_rejected("invalid token", false);
    assert!(msg.contains("identity provider rejected"), "{msg}");
    assert!(msg.contains("manta config unset auth"), "{msg}");
    assert!(msg.contains("invalid token"), "{msg}");
  }

  /// `config unset auth` does nothing for a service account.
  #[test]
  fn a_401_with_a_service_account_points_at_it() {
    let msg = token_rejected("invalid token", true);
    assert!(msg.contains("MANTA_SA_CLIENT_ID"), "{msg}");
    assert!(!msg.contains("config unset auth"), "{msg}");
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

  #[test]
  fn extract_error_body_parses_error_field() {
    let body = r#"{"error":"Not found: Image 'abc-123'"}"#;
    assert_eq!(extract_error_body(body, 404), "Not found: Image 'abc-123'");
  }

  #[test]
  fn extract_error_body_returns_raw_on_non_json() {
    let body = "plain-text failure";
    assert_eq!(extract_error_body(body, 500), body);
  }

  #[test]
  fn extract_error_body_flags_empty_body() {
    assert_eq!(
      extract_error_body("", 502),
      "<empty response body, status 502>"
    );
  }
}

/// Pre-request hook of the generated client (see `build.rs`): give the
/// request a new token. Without a provider the request goes out
/// unauthenticated.
///
/// # Errors
///
/// No token could be obtained ([`TokenProvider::token`]); the request is
/// not sent. The message, with its causes, reaches the user through
/// [`OpenApiResultExt::into_anyhow`].
pub(crate) async fn set_current_token(
  provider: &Option<Arc<TokenProvider>>,
  request: &mut reqwest::Request,
) -> Result<(), String> {
  let Some(provider) = provider else {
    return Ok(());
  };
  let token = provider.token().await.map_err(|e| format!("{e:#}"))?;
  let bearer = bearer_header(&token).map_err(|e| format!("{e:#}"))?;
  request
    .headers_mut()
    .insert(reqwest::header::AUTHORIZATION, bearer);
  Ok(())
}

/// `Authorization: Bearer <token>`, marked sensitive so it is never
/// logged.
fn bearer_header(token: &str) -> anyhow::Result<HeaderValue> {
  let mut bearer = HeaderValue::from_str(&format!("Bearer {token}"))
    .context("the token is not a valid HTTP header value")?;
  bearer.set_sensitive(true);
  Ok(bearer)
}

/// HTTP client that forwards CLI requests to a manta server.
///
/// Wraps three things:
/// - the progenitor-generated `crate::openapi_client::Client` (the
///   `openapi` field) for every endpoint declared in `openapi.json`;
/// - a `reqwest::Client` (`raw`) sharing the same bearer-auth default
///   header, used by the hand-rolled WebSocket / SSE paths;
/// - the base URL (`<scheme>://host:port/v2`) and the
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
/// use crate::http_client::{MantaClient, OpenApiResultExt};
///
/// let client = MantaClient::from_app_ctx(ctx)?;
/// let groups = client
///   .openapi
///   .get_groups(None, client.site_name()?)
///   .await
///   .into_anyhow()
///   .await?;
/// ```
#[derive(Debug)]
pub struct MantaClient {
  /// Generated typed API client. Dispatch handlers call
  /// `client.openapi.<method>(...).await.into_anyhow().await?`.
  pub openapi: crate::openapi_client::Client,
  /// `X-Manta-Site` header value the server uses to pick the backend
  /// config. Threaded explicitly to every `openapi.*` call. Set from
  /// `--site` / `cli.toml` at construction, or — for a client built
  /// by [`Self::from_app_ctx_resolving`] without one — from the first
  /// response whose `X-Manta-Site` header names the site the server
  /// resolved ([`OpenApiResultExt::into_anyhow_pinning_site`]).
  site: std::sync::OnceLock<String>,
  /// Plain `reqwest::Client` for the WS / SSE paths, which add the
  /// token themselves ([`Self::bearer_token`]).
  pub raw: reqwest::Client,
  /// Supplies every request's token; `None` for an unauthenticated
  /// client.
  token_provider: Option<Arc<TokenProvider>>,
  /// Base URL `<http(s)://host:port>/v2` used by the WS / SSE
  /// paths to build `wss://…` and `…/sessions/{name}/logs` URLs.
  pub base_url: String,
}

impl MantaClient {
  /// Build a client from an `AppContext`, honouring its
  /// `request_timeout_secs` (loaded from `cli.toml`).
  ///
  /// Requests carry tokens from `ctx.token_provider` (set by
  /// `process_cli`).
  ///
  /// # Errors
  ///
  /// - `ctx.require_site()` fails when no site is set.
  /// - Either internal `reqwest::Client::build` fails.
  pub fn from_app_ctx(
    ctx: &crate::common::app_context::AppContext<'_>,
  ) -> anyhow::Result<Self> {
    Self::new_with_timeout(
      ctx.manta_server_url,
      Some(ctx.require_site()?),
      ctx.request_timeout_secs,
      ctx.token_provider.clone(),
    )
  }

  /// Like [`Self::from_app_ctx`], but without requiring a site: for
  /// commands whose first request names a group or xname list the
  /// server can resolve the site from. Send that request with
  /// [`Self::site_hint`] and finish it with
  /// [`OpenApiResultExt::into_anyhow_pinning_site`]; later calls can
  /// then use [`Self::site_name`].
  ///
  /// # Errors
  ///
  /// As [`Self::from_app_ctx`], minus the missing-site case.
  pub fn from_app_ctx_resolving(
    ctx: &crate::common::app_context::AppContext<'_>,
  ) -> anyhow::Result<Self> {
    Self::new_with_timeout(
      ctx.manta_server_url,
      ctx.site_name,
      ctx.request_timeout_secs,
      ctx.token_provider.clone(),
    )
  }

  /// Build a client with an explicit optional per-request timeout
  /// and optional token provider.
  ///
  /// `token_provider`, when set, supplies a new token for each generated
  /// call (in its pre-request hook) and each raw connection.
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
  /// If `server_url` has no scheme, `http://` is prepended. This lets users
  /// write `manta_server_url = "localhost:8080"` in their config without
  /// triggering a "URL scheme is not allowed" error from reqwest.
  ///
  /// # Errors
  ///
  /// - Either internal `reqwest::Client::build` fails (TLS init,
  ///   resolver init, etc.).
  pub fn new_with_timeout(
    server_url: &str,
    site_name: Option<&str>,
    timeout_secs: Option<u64>,
    token_provider: Option<Arc<TokenProvider>>,
  ) -> anyhow::Result<Self> {
    let normalized = if server_url.starts_with("http://")
      || server_url.starts_with("https://")
    {
      server_url.to_owned()
    } else {
      format!("http://{server_url}")
    };
    let base_url = format!("{}/v2", normalized.trim_end_matches('/'));

    // Streams + WebSockets keep the original Option<u64> semantics
    // (None = no timeout).
    let mut raw_builder = reqwest::Client::builder();
    if let Some(secs) = timeout_secs {
      raw_builder = raw_builder.timeout(std::time::Duration::from_secs(secs));
    }
    let raw = raw_builder.build().context("Failed to build HTTP client")?;

    // One-shot API calls default to 5 minutes when nothing is
    // configured. An explicit cli.toml override wins.
    let api_timeout_secs = timeout_secs.unwrap_or(DEFAULT_API_TIMEOUT_SECS);
    let openapi_inner = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(api_timeout_secs))
      .build()
      .context("Failed to build OpenAPI HTTP client")?;
    let openapi = crate::openapi_client::Client::new_with_client(
      &base_url,
      openapi_inner,
      token_provider.clone(),
    );

    let site = std::sync::OnceLock::new();
    if let Some(name) = site_name {
      let _ = site.set(name.to_owned());
    }
    Ok(Self {
      openapi,
      site,
      raw,
      token_provider,
      base_url,
    })
  }

  /// A new token for a raw connection; `None` for an unauthenticated
  /// client.
  ///
  /// # Errors
  ///
  /// No token could be obtained ([`TokenProvider::token`]).
  pub(crate) async fn bearer_token(&self) -> anyhow::Result<Option<String>> {
    match &self.token_provider {
      Some(provider) => provider.token().await.map(Some),
      None => Ok(None),
    }
  }

  /// The token sent last, for reading its claims; `None` before the
  /// first request or for an unauthenticated client.
  pub(crate) fn last_token(&self) -> Option<String> {
    self.token_provider.as_ref()?.last_issued()
  }

  /// Site name for the `openapi.<method>(args, client.site_name()?)`
  /// calls whose endpoint requires `X-Manta-Site`.
  ///
  /// # Errors
  ///
  /// No site was configured and none has been resolved by the server
  /// yet.
  pub fn site_name(&self) -> anyhow::Result<&str> {
    self.site_hint().ok_or_else(|| {
      anyhow::anyhow!(
        "No site selected. Pass --site <name> or set `site` in cli.toml"
      )
    })
  }

  /// Site name if known, for the calls whose endpoint can resolve the
  /// site itself when `X-Manta-Site` is omitted.
  pub fn site_hint(&self) -> Option<&str> {
    self.site.get().map(String::as_str)
  }

  /// Adopt the site the server resolved, as named by a response's
  /// `X-Manta-Site` header, and tell the user once. A no-op when the
  /// site is already known (the server only echoes resolved sites, so
  /// this is the first resolution).
  pub fn pin_site(&self, headers: &reqwest::header::HeaderMap) {
    let Some(site) = headers.get("X-Manta-Site").and_then(|v| v.to_str().ok())
    else {
      return;
    };
    if self.site.set(site.to_owned()).is_ok() {
      eprintln!("site '{site}' resolved by manta-server");
      tracing::info!(site, "site resolved by manta-server");
    }
  }

  /// Base URL `<scheme>://<host>/v2` — used by the WebSocket /
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
    let server_url = self.base_url.trim_end_matches("/v2");
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
