//! Public-router auth handlers (`POST /api/v1/auth/{token,validate}`).
//!
//! Deliberately not behind the `BearerToken` extractor — these are the
//! endpoints clients call *to obtain* a bearer token. The defensive
//! middleware (rate limit, body redaction) lives in
//! `crate::server::auth_middleware`; this file just maps requests to
//! [`crate::service::auth`].
//!
//! An unknown `X-Manta-Site` is reported explicitly as `404 Not Found`
//! so the CLI can fail fast instead of prompting for credentials that
//! can never succeed. This is a deliberate trade-off: because `/auth/*`
//! takes no bearer token, it lets an *unauthenticated* caller tell a
//! configured site (401) from an unknown one (404) — reversing this
//! module's former "reveal nothing about site config" stance. It is
//! considered acceptable because site names are not secrets and the
//! per-IP rate limiter in [`crate::server::auth_middleware`] bounds
//! enumeration. (The authenticated endpoints already expose the same
//! distinction once *any* syntactically-valid bearer header is present,
//! since [`crate::server::handlers::RequestCtx`] does the site lookup
//! before the token is validated against the backend.)
//!
//! Genuine credential rejections surface a generic `401 invalid
//! credentials` so the response never reveals whether a username
//! exists or what the backend actually rejected. On the *token
//! exchange*, failures where the backend never evaluated the
//! credentials at all — unreachable, timed out, or answering 5xx —
//! are split out as `502`/`504` gateway errors with `MANTA_BACKEND_*`
//! codes (see [`auth_failure_response`]); they carry no credential
//! information, so the anti-enumeration stance is preserved.
//! `auth_validate` cannot make the same split: csm-rs pre-stringifies
//! its transport failures, so a backend outage during token
//! *validation* still collapses into the generic 401 (the
//! known-limitation note in `ERRORS.md`; typed upstream errors are
//! issue #107). The specific reason is captured server-side with
//! `tracing::warn!` and sent to the audit channel when one is
//! configured on [`ServerState`].

use std::net::SocketAddr;
use std::sync::Arc;

use crate::server::common::audit;
use axum::{
  Json,
  extract::{ConnectInfo, State},
  http::StatusCode,
  response::IntoResponse,
};
use manta_shared::types::auth::{
  AuthTokenRequest, AuthTokenResponse, ValidateTokenRequest,
};

use manta_backend_dispatcher::error::Error as BackendError;
use manta_shared::common::error_code::ErrorCode;

use super::{ErrorResponse, ServerState, SiteHeader, SiteName};
use crate::service;

/// Single generic 401 surfaced to clients for any `/auth/*`
/// *credential* failure. Detail stays server-side in `tracing::warn!`.
fn generic_invalid_credentials() -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::UNAUTHORIZED,
    Json(ErrorResponse::new(
      ErrorCode::InvalidCredentials,
      "invalid credentials",
    )),
  )
}

/// Classify a failed credential exchange.
///
/// Genuine rejections stay the deliberately generic 401 above
/// (anti-enumeration policy — no hint whether the user exists). But a
/// failure where the backend never *evaluated* the credentials —
/// unreachable, timed out, or answering 5xx — must not be labelled
/// `MANTA_INVALID_CREDENTIALS`: a script branching on the code would
/// misread an outage as a bad password, and the CLI would re-prompt
/// for credentials that were never checked. Those cases become
/// gateway errors with an honest `MANTA_BACKEND_*` code and a message
/// that carries no credential information.
///
/// The typed split works because csm-rs's Keycloak exchange surfaces
/// transport failures and non-2xx statuses as
/// `BackendError::NetError(reqwest::Error)` (`.send().await?` /
/// `.error_for_status()?`). Upstream failures that arrive
/// pre-stringified (e.g. `Message(...)`) can't be classified and fall
/// back to the generic 401 — see `ERRORS.md` and issue #107.
fn auth_failure_response(
  e: &BackendError,
) -> (StatusCode, Json<ErrorResponse>) {
  match e {
    BackendError::NetError(rqe) if rqe.is_timeout() => (
      StatusCode::GATEWAY_TIMEOUT,
      Json(ErrorResponse::new(
        ErrorCode::BackendTimeout,
        "authentication backend did not respond in time; \
         credentials were not checked",
      )),
    ),
    // A status-bearing reqwest error means the backend answered.
    // 4xx = it evaluated and rejected the credentials → generic 401.
    // 5xx = it is erroring → honest gateway error.
    BackendError::NetError(rqe) if rqe.status().is_some() => {
      let status = rqe.status().expect("guard checked is_some");
      if status.is_server_error() {
        (
          StatusCode::BAD_GATEWAY,
          Json(ErrorResponse::new(
            ErrorCode::BackendHttpError,
            format!(
              "authentication backend returned {status}; \
               credentials were not checked"
            ),
          )),
        )
      } else {
        generic_invalid_credentials()
      }
    }
    // No status and no timeout: the request never completed —
    // connect refused, DNS failure, TLS failure, or a dropped
    // connection mid-request.
    BackendError::NetError(rqe) => (
      StatusCode::BAD_GATEWAY,
      Json(ErrorResponse::new(
        if rqe.is_connect() {
          ErrorCode::BackendConnectFailed
        } else {
          ErrorCode::NetworkError
        },
        "could not reach the authentication backend; \
         credentials were not checked",
      )),
    ),
    _ => generic_invalid_credentials(),
  }
}

/// `404` returned when the `X-Manta-Site` header names a site that is
/// not configured on this server. Unlike credential failures (which
/// stay a generic 401), an unknown site is reported explicitly so the
/// CLI can fail fast instead of prompting for credentials that can
/// never succeed. This intentionally reveals site existence to
/// unauthenticated callers — see the module-level docs for the
/// trade-off.
fn site_not_found(site: &str) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::NOT_FOUND,
    Json(ErrorResponse::new(
      ErrorCode::SiteNotFound,
      format!("site '{site}' not found"),
    )),
  )
}

/// POST /api/v1/auth/token — exchange username/password for a CSM token.
#[utoipa::path(post, path = "/auth/token", tag = "auth",
  params(SiteHeader),
  request_body = AuthTokenRequest,
  responses(
    (status = 200, description = "Token issued", body = AuthTokenResponse),
    (status = 401, description = "Invalid credentials", body = ErrorResponse),
    (status = 404, description = "Unknown site", body = ErrorResponse),
    (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
    (status = 502, description = "Authentication backend unreachable or erroring; credentials were not checked", body = ErrorResponse),
    (status = 504, description = "Authentication backend timed out; credentials were not checked", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn auth_token(
  State(state): State<Arc<ServerState>>,
  SiteName(site_name): SiteName,
  ConnectInfo(peer): ConnectInfo<SocketAddr>,
  Json(req): Json<AuthTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(|e| {
    tracing::warn!("auth_token: site lookup failed: {}", e);
    site_not_found(&site_name)
  })?;
  let source_ip = peer.ip().to_string();

  tracing::info!(
    user = %req.username,
    site = %site_name,
    from = %source_ip,
    "auth_token: credential exchange requested"
  );

  match service::auth::get_api_token(&infra, &req.username, &req.password).await
  {
    Ok(token) => {
      tracing::info!(
        user = %req.username,
        site = %site_name,
        from = %source_ip,
        "auth_token: token issued"
      );
      audit::send_auth_audit(
        state.auditor.as_ref(),
        "success",
        &req.username,
        &source_ip,
        &site_name,
      )
      .await;
      Ok(Json(AuthTokenResponse { token }))
    }
    Err(e) => {
      tracing::warn!(
        "auth_token: backend rejected user={} site={} from={}: {}",
        req.username,
        site_name,
        source_ip,
        e
      );
      audit::send_auth_audit(
        state.auditor.as_ref(),
        "failure",
        &req.username,
        &source_ip,
        &site_name,
      )
      .await;
      Err(auth_failure_response(&e))
    }
  }
}

/// POST /api/v1/auth/validate — check whether a CSM token is still valid.
#[utoipa::path(post, path = "/auth/validate", tag = "auth",
  params(SiteHeader),
  request_body = ValidateTokenRequest,
  responses(
    (status = 200, description = "Token is valid"),
    (status = 401, description = "Token rejected", body = ErrorResponse),
    (status = 404, description = "Unknown site", body = ErrorResponse),
    (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn auth_validate(
  State(state): State<Arc<ServerState>>,
  SiteName(site_name): SiteName,
  Json(req): Json<ValidateTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(|e| {
    tracing::warn!("auth_validate: site lookup failed: {}", e);
    site_not_found(&site_name)
  })?;
  tracing::info!(site = %site_name, "auth_validate: token check requested");
  match service::auth::validate_api_token(&infra, &req.token).await {
    Ok(()) => {
      tracing::info!(site = %site_name, "auth_validate: token accepted");
      Ok(StatusCode::OK)
    }
    Err(e) => {
      tracing::warn!("auth_validate: backend rejected token: {}", e);
      Err(generic_invalid_credentials())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Build a status-bearing `reqwest::Error` the same way csm-rs's
  /// Keycloak exchange produces one: a response run through
  /// `error_for_status()`.
  fn reqwest_status_error(status: u16) -> reqwest::Error {
    let http_resp = axum::http::Response::builder()
      .status(status)
      .body("boom".to_string())
      .expect("valid response");
    reqwest::Response::from(http_resp)
      .error_for_status()
      .expect_err("status is an error")
  }

  #[test]
  fn backend_5xx_becomes_502_backend_http_error() {
    let e = BackendError::NetError(reqwest_status_error(503));
    let (status, axum::Json(body)) = auth_failure_response(&e);
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body.code.as_deref(), Some("MANTA_BACKEND_HTTP_ERROR"));
    assert!(body.error.contains("credentials were not checked"));
  }

  #[test]
  fn backend_401_stays_generic_invalid_credentials() {
    // Keycloak answered 401: it evaluated and rejected the
    // credentials — the anti-enumeration 401 must be preserved.
    let e = BackendError::NetError(reqwest_status_error(401));
    let (status, axum::Json(body)) = auth_failure_response(&e);
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body.code.as_deref(), Some("MANTA_INVALID_CREDENTIALS"));
    assert_eq!(body.error, "invalid credentials");
  }

  #[test]
  fn stringly_failures_fall_back_to_generic_401() {
    // Upstream errors that arrive pre-stringified can't be classified
    // (issue #107); they must keep today's behaviour, not 502.
    let e = BackendError::Message("CSM-RS > Net: something".into());
    let (status, axum::Json(body)) = auth_failure_response(&e);
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body.code.as_deref(), Some("MANTA_INVALID_CREDENTIALS"));
  }
}
