//! Runtime-configuration handler.
//!
//! - `PUT /v2/runtime-configuration` → [`apply_runtime_configuration`]
//!
//! Thin wrapper around
//! [`crate::service::runtime_configuration::apply_runtime_configuration`].

use axum::{Json, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

pub use manta_shared::types::api::runtime_configuration::ApplyRuntimeConfigurationRequest;

/// `PUT /v2/runtime-configuration` — assign a CFS configuration as
/// the desired runtime configuration for a set of nodes.
#[utoipa::path(put, path = "/runtime-configuration", tag = "runtime-configuration",
  params(SiteHeader),
  request_body = ApplyRuntimeConfigurationRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Runtime configuration applied",  body = serde_json::Value),
    (status = 400, description = "Bad request",                    body = ErrorResponse),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 403, description = "Forbidden",                      body = ErrorResponse),
    (status = 404, description = "Not found",                      body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn apply_runtime_configuration(
  ctx: RequestCtx,
  Json(body): Json<ApplyRuntimeConfigurationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "apply_runtime_configuration cfs_configuration_name={} hosts={} enabled={} dry_run={}",
    body.cfs_configuration_name,
    body.hosts_expression,
    body.enabled,
    body.dry_run
  );
  let infra = ctx.infra();

  let xnames = service::runtime_configuration::apply_runtime_configuration(
    &infra,
    &ctx.token,
    &body.cfs_configuration_name,
    &body.hosts_expression,
    body.enabled,
    body.dry_run,
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "dry_run": body.dry_run,
      "cfs_configuration_name": body.cfs_configuration_name,
      "enabled": body.enabled,
      "nodes": xnames,
    })),
  ))
}
