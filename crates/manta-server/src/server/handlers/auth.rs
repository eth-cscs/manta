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
//! Every *other* auth failure surfaces a generic `401 invalid
//! credentials` so the response never reveals whether a username
//! exists or what the backend actually rejected. The specific reason
//! is captured server-side with `tracing::warn!` and sent to the
//! audit channel when one is configured on [`ServerState`].

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

use super::{ErrorResponse, ServerState, SiteHeader, SiteName};
use crate::service;

/// Single generic 401 surfaced to clients for any `/auth/*`
/// *credential* failure. Detail stays server-side in `tracing::warn!`.
fn generic_invalid_credentials() -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::UNAUTHORIZED,
    Json(ErrorResponse {
      error: "invalid credentials".to_string(),
    }),
  )
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
    Json(ErrorResponse {
      error: format!("site '{site}' not found"),
    }),
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
      Err(generic_invalid_credentials())
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
