//! Top-level Axum handlers module.
//!
//! `mod.rs` keeps:
//! - request extractors (`BearerToken`, `SiteName`, `RequestCtx`, `SiteHeader`)
//! - the `ErrorResponse` body type + error mappers (`to_handler_error`,
//!   `display_error`, `serialize_or_500`)
//! - guard helpers (`require_vault`, `require_k8s_url`,
//!   `validate_repo_list_lengths`, `parse_iso_datetime`)
//! - the cross-handler `resolve_xnames_from_request` helper
//! - the `health` endpoint
//!
//! Every other handler lives in a per-resource sub-module (mirroring
//! the `service/` layout) and is re-exported here so `routes.rs` and
//! `api_doc.rs` can keep referencing `handlers::X` unchanged.

use std::sync::Arc;

use axum::{
  Json,
  extract::FromRequestParts,
  http::{StatusCode, header, request::Parts},
  response::IntoResponse,
};
use manta_backend_dispatcher::error::Error as BackendError;
use serde::Serialize;
use utoipa::{IntoParams, ToSchema};

use super::ServerState;

mod auth;
mod boot_parameters;
mod cluster;
mod configuration;
mod console;
mod ephemeral_env;
mod group;
mod hardware;
mod hw_cluster;
mod image;
mod kernel_parameters;
mod migrate;
mod node;
mod power;
mod redfish_endpoints;
mod sat_file;
mod session;
mod template;

pub use auth::*;
pub use boot_parameters::*;
pub use cluster::*;
pub use configuration::*;
pub use console::*;
pub use ephemeral_env::*;
pub use group::*;
pub use hardware::*;
pub use hw_cluster::*;
pub use image::*;
pub use kernel_parameters::*;
pub use migrate::*;
pub use node::*;
pub use power::*;
pub use redfish_endpoints::*;
pub use sat_file::*;
pub use session::*;
pub use template::*;

// ---------------------------------------------------------------------------
// Bearer-token extractor — eliminates token-extraction boilerplate
// ---------------------------------------------------------------------------

/// Axum extractor that pulls the token from `Authorization: Bearer <token>`.
pub struct BearerToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for BearerToken {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let auth_header = parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Missing Authorization header".to_string(),
          }),
        )
      })?;

    let token = auth_header
      .strip_prefix("Bearer ")
      .or_else(|| auth_header.strip_prefix("bearer "))
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Authorization header must use Bearer scheme".to_string(),
          }),
        )
      })?;

    Ok(BearerToken(token.to_string()))
  }
}

/// Axum extractor that reads the target site name from `X-Manta-Site`.
///
/// Every handler that touches backend APIs requires this header so the server
/// knows which site's CA certificate, base URL, and credentials to use.
pub struct SiteName(pub String);

impl<S: Send + Sync> FromRequestParts<S> for SiteName {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let site = parts
      .headers
      .get("X-Manta-Site")
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::BAD_REQUEST,
          Json(ErrorResponse {
            error: "Missing X-Manta-Site header".to_string(),
          }),
        )
      })?;
    Ok(SiteName(site.to_string()))
  }
}

/// Required header parameter present on every authenticated endpoint.
///
/// Tells the server which cluster to route the request to.
/// **Not** an authentication mechanism — documented as a plain header parameter.
///
/// The field is consumed by the `utoipa::IntoParams` derive macro at compile
/// time to generate the OpenAPI spec; the runtime extractor is [`SiteName`].
#[derive(IntoParams)]
#[into_params(parameter_in = Header)]
#[allow(dead_code)]
pub struct SiteHeader {
  /// Name of the target cluster (matches a site configured in the server).
  #[param(required = true, rename = "X-Manta-Site")]
  pub x_manta_site: String,
}

// ---------------------------------------------------------------------------
// RequestCtx — bundles the State + BearerToken + SiteName extractors that
// every authenticated handler opens with. Plus `infra()` for the
// `state.infra_context(&site_name).map_err(to_handler_error)?` line that
// follows. Each handler shrinks by 3-4 lines.
// ---------------------------------------------------------------------------

/// Bundled extractor for `State<Arc<ServerState>>` + `BearerToken` +
/// `SiteName`. Use it in handler signatures instead of the three
/// individual extractors when all three are needed (the typical case).
///
/// The unauthenticated `/auth/*` handlers and the health endpoint
/// still use explicit extractors — they don't need a Bearer token.
pub struct RequestCtx {
  pub state: Arc<ServerState>,
  pub token: String,
  pub site_name: String,
}

impl FromRequestParts<Arc<ServerState>> for RequestCtx {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    state: &Arc<ServerState>,
  ) -> Result<Self, Self::Rejection> {
    let BearerToken(token) =
      BearerToken::from_request_parts(parts, state).await?;
    let SiteName(site_name) =
      SiteName::from_request_parts(parts, state).await?;
    Ok(Self {
      state: Arc::clone(state),
      token,
      site_name,
    })
  }
}

impl RequestCtx {
  /// Consume into `(state, token, site_name)` for the
  /// `let (state, token, site_name) = ctx.into_parts();` opening every
  /// authenticated handler uses. Tuple form keeps `rustfmt` from
  /// expanding the destructure across five lines.
  pub fn into_parts(self) -> (Arc<ServerState>, String, String) {
    (self.state, self.token, self.site_name)
  }
}

/// Convert a `BackendError` into the best-fitting HTTP error response.
///
/// `pub` (rather than `pub(crate)`) so the integration tests in
/// `crates/manta-server/tests/` can exercise the mapping directly.
pub fn to_handler_error(e: BackendError) -> (StatusCode, Json<ErrorResponse>) {
  let status = match &e {
    BackendError::NotFound(_)
    | BackendError::SessionNotFound
    | BackendError::ConfigurationNotFound => StatusCode::NOT_FOUND,
    BackendError::Conflict(_)
    | BackendError::ConfigurationAlreadyExistsError(_) => StatusCode::CONFLICT,
    BackendError::BadRequest(_)
    | BackendError::InvalidPattern(_)
    | BackendError::UnsupportedBackend(_)
    | BackendError::InvalidNodeId(_) => StatusCode::BAD_REQUEST,
    BackendError::AuthenticationTokenNotFound(_)
    | BackendError::JwtMalformed(_) => StatusCode::UNAUTHORIZED,
    BackendError::InsufficientResources(_) => StatusCode::UNPROCESSABLE_ENTITY,
    _ => StatusCode::INTERNAL_SERVER_ERROR,
  };
  if status == StatusCode::INTERNAL_SERVER_ERROR {
    tracing::error!("Internal error: {}", e);
  } else {
    tracing::debug!("Service error {}: {}", status, e);
  }
  (
    status,
    Json(ErrorResponse {
      error: e.to_string(),
    }),
  )
}

/// Convert any `Display` error (e.g. anyhow) into an HTTP error response.
pub(super) fn display_error<E: std::fmt::Display>(
  e: E,
) -> (StatusCode, Json<ErrorResponse>) {
  to_handler_error(BackendError::Message(e.to_string()))
}

pub(super) fn serialize_or_500<T: Serialize>(
  v: &T,
) -> Result<serde_json::Value, (StatusCode, Json<ErrorResponse>)> {
  serde_json::to_value(v).map_err(|e| {
    let msg = format!("Failed to serialize: {}", e);
    tracing::error!("{}", msg);
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse { error: msg }),
    )
  })
}

pub(super) fn require_vault(
  url: Option<&str>,
) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "vault_base_url not configured on this server".into(),
      }),
    )
  })
}

pub(super) fn require_k8s_url(
  url: Option<&str>,
) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "k8s_api_url not configured on this server".into(),
      }),
    )
  })
}

pub(super) fn validate_repo_list_lengths(
  repo_names: &[String],
  repo_last_commit_ids: &[String],
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
  if repo_names.len() != repo_last_commit_ids.len() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!(
          "repo_names ({}) and repo_last_commit_ids ({}) must have the same length",
          repo_names.len(),
          repo_last_commit_ids.len()
        ),
      }),
    ));
  }
  Ok(())
}

pub(super) fn default_true() -> bool {
  true
}

pub(super) fn parse_iso_datetime(
  field: &str,
  value: &str,
) -> Result<chrono::NaiveDateTime, (StatusCode, Json<ErrorResponse>)> {
  chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").map_err(
    |e| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid '{}' datetime '{}': {}", field, value, e),
        }),
      )
    },
  )
}

// ---------------------------------------------------------------------------
// Shared response types
// ---------------------------------------------------------------------------

/// Standard JSON error body returned by all failed endpoints.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
  pub error: String,
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// GET /health — liveness probe; returns `{"status":"ok"}`.
#[utoipa::path(get, path = "/health", tag = "system",
  responses(
    (status = 200, description = "Server is healthy"),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn health() -> impl IntoResponse {
  Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolve target xnames from an explicit list or an HSM group name.
/// Returns 400 if neither is provided.
async fn resolve_xnames_from_request(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  token: &str,
  xnames_expression: Option<&str>,
  hsm_group: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
  if let Some(expr) = xnames_expression
    && !expr.is_empty()
  {
    return crate::server::common::node_ops::resolve_hosts_expression(
      backend, token, expr, false,
    )
    .await
    .map_err(display_error);
  }
  if let Some(group) = hsm_group {
    return crate::server::common::node_ops::resolve_target_nodes(
      backend,
      token,
      None,
      Some(group),
      None,
    )
    .await
    .map_err(display_error);
  }
  Err((
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse {
      error: "At least one of 'xnames' or 'hsm_group' must be provided"
        .to_string(),
    }),
  ))
}
