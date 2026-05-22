//! Hardware cluster add/delete/configuration handlers.

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, default_true, display_error,
  to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// POST /api/v1/hardware-clusters/{target}/members
// ---------------------------------------------------------------------------

/// Request body for `POST /hardware-clusters/{target}/members`.
#[derive(Deserialize, ToSchema)]
pub struct AddHwComponentRequest {
  /// Source HSM group that donates nodes matching `pattern`.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move.
  pub pattern: String,
  /// Create the target HSM group if it does not already exist.
  #[serde(default)]
  pub create_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/hardware-clusters/{target}/members` — move nodes matching a hardware pattern into a cluster.
#[utoipa::path(post, path = "/hardware-clusters/{target}/members", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = AddHwComponentRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Members added or preview", body = serde_json::Value),
    (status = 401, description = "Unauthorized",             body = ErrorResponse),
    (status = 500, description = "Internal error",           body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_hw_component(
  ctx: RequestCtx,
  Path(target): Path<String>,
  Json(body): Json<AddHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "add_hw_component target={} parent={} dry_run={}",
    target,
    body.parent_cluster,
    body.dry_run
  );
  let infra = ctx.infra();

  service::group::validate_hsm_group_access(&infra, &ctx.token, &target)
    .await
    .map_err(to_handler_error)?;
  service::group::validate_hsm_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let result = crate::service::hw_cluster::add_hw_component(
    infra.backend,
    &ctx.token,
    &target,
    &body.parent_cluster,
    &body.pattern,
    body.dry_run,
    body.create_hsm_group,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "nodes_moved": result.nodes_moved,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/hardware-clusters/{target}/members
// ---------------------------------------------------------------------------

/// Request body for `DELETE /hardware-clusters/{target}/members`.
#[derive(Deserialize, ToSchema)]
pub struct DeleteHwComponentRequest {
  /// Destination HSM group that receives nodes moved out of the target cluster.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move back.
  pub pattern: String,
  /// Delete the target HSM group if it becomes empty after the operation.
  #[serde(default)]
  pub delete_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/hardware-clusters/{target}/members` — move nodes back to parent cluster by hardware pattern.
#[utoipa::path(delete, path = "/hardware-clusters/{target}/members", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = DeleteHwComponentRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Members removed or preview", body = serde_json::Value),
    (status = 401, description = "Unauthorized",               body = ErrorResponse),
    (status = 500, description = "Internal error",             body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_hw_component(
  ctx: RequestCtx,
  Path(target): Path<String>,
  Json(body): Json<DeleteHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "delete_hw_component target={} parent={} dry_run={}",
    target,
    body.parent_cluster,
    body.dry_run
  );
  let infra = ctx.infra();

  service::group::validate_hsm_group_access(&infra, &ctx.token, &target)
    .await
    .map_err(to_handler_error)?;
  service::group::validate_hsm_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let result = crate::service::hw_cluster::delete_hw_component(
    infra.backend,
    &ctx.token,
    &target,
    &body.parent_cluster,
    &body.pattern,
    body.dry_run,
    body.delete_hsm_group,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "nodes_moved": result.nodes_moved,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/hardware-clusters/{target}/configuration
// ---------------------------------------------------------------------------

/// Whether to pin nodes to the target cluster or unpin them back to the parent.
#[derive(Debug, Deserialize, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HwClusterMode {
  /// Move nodes from the parent cluster into the target cluster.
  #[default]
  Pin,
  /// Move nodes back from the target cluster to the parent cluster.
  Unpin,
}

/// Request body for `POST /hardware-clusters/{target}/configuration`.
#[derive(Deserialize, ToSchema)]
pub struct ApplyHwConfigurationRequest {
  /// Source (parent) HSM group supplying nodes.
  pub parent_cluster: String,
  /// Hardware component pattern selecting which nodes to pin/unpin.
  pub pattern: String,
  /// Whether to pin nodes into the target cluster or unpin back to parent (default: pin).
  #[serde(default)]
  pub mode: HwClusterMode,
  /// Create the target HSM group if absent (default true).
  #[serde(default = "default_true")]
  pub create_target_hsm_group: bool,
  /// Delete the parent HSM group if it becomes empty (default true).
  #[serde(default = "default_true")]
  pub delete_empty_parent_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/hardware-clusters/{target}/configuration` — pin or unpin nodes between clusters by hardware pattern.
#[utoipa::path(post, path = "/hardware-clusters/{target}/configuration", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = ApplyHwConfigurationRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Configuration applied or preview", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                     body = ErrorResponse),
    (status = 500, description = "Internal error",                   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn apply_hw_configuration(
  ctx: RequestCtx,
  Path(target): Path<String>,
  Json(body): Json<ApplyHwConfigurationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "apply_hw_configuration target={} parent={} dry_run={}",
    target,
    body.parent_cluster,
    body.dry_run
  );
  let infra = ctx.infra();

  service::group::validate_hsm_group_access(&infra, &ctx.token, &target)
    .await
    .map_err(to_handler_error)?;
  service::group::validate_hsm_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let mode = match body.mode {
    HwClusterMode::Pin => crate::service::hw_cluster::HwClusterMode::Pin,
    HwClusterMode::Unpin => crate::service::hw_cluster::HwClusterMode::Unpin,
  };

  let result = crate::service::hw_cluster::apply_hw_configuration(
    infra.backend,
    mode,
    &ctx.token,
    &target,
    &body.parent_cluster,
    &body.pattern,
    body.dry_run,
    body.create_target_hsm_group,
    body.delete_empty_parent_hsm_group,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}
