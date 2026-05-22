//! Public-router auth handlers (`POST /api/v1/auth/{token,validate}`).
//!
//! Deliberately not behind the `BearerToken` extractor — these are the
//! endpoints clients call *to obtain* a bearer token. The defensive
//! middleware (rate limit, body redaction) lives in
//! `crate::server::auth_middleware`; this file just maps requests to
//! `service::auth`.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
  Json,
  extract::{ConnectInfo, State},
  http::StatusCode,
  response::IntoResponse,
};
use manta_shared::common::audit;
use manta_shared::shared::auth::{
  AuthTokenRequest, AuthTokenResponse, ValidateTokenRequest,
};

use super::{ErrorResponse, ServerState, SiteHeader, SiteName};
use crate::service;

/// Single generic 401 surfaced to clients for any `/auth/*` failure.
/// Detail stays server-side in `tracing::warn!`.
fn generic_invalid_credentials() -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::UNAUTHORIZED,
    Json(ErrorResponse {
      error: "invalid credentials".to_string(),
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
    generic_invalid_credentials()
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
    generic_invalid_credentials()
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
