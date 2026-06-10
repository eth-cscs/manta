//! Hardware cluster add/delete/configuration handlers.

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

pub use manta_shared::types::wire::hw_cluster::{
  AddHwComponentRequest, ApplyHwConfigurationRequest, DeleteHwComponentRequest,
  HwClusterMode,
};

/// `POST /api/v1/hardware-clusters/{target}/members` — move nodes matching a hardware pattern into a cluster.
#[utoipa::path(post, path = "/hardware-clusters/{target}/members", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = AddHwComponentRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
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

  service::authorization::validate_user_group_access(
    &infra, &ctx.token, &target,
  )
  .await
  .map_err(to_handler_error)?;
  service::authorization::validate_user_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let result = crate::service::hw_cluster::add_hw_component(
    &infra,
    &ctx.token,
    &target,
    &body.parent_cluster,
    &body.pattern,
    body.dry_run,
    body.create_hsm_group,
  )
  .await
  .map_err(to_handler_error)?;

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

/// `DELETE /api/v1/hardware-clusters/{target}/members` — move nodes back to parent cluster by hardware pattern.
#[utoipa::path(delete, path = "/hardware-clusters/{target}/members", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = DeleteHwComponentRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
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

  service::authorization::validate_user_group_access(
    &infra, &ctx.token, &target,
  )
  .await
  .map_err(to_handler_error)?;
  service::authorization::validate_user_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let result = crate::service::hw_cluster::delete_hw_component(
    &infra,
    &ctx.token,
    &target,
    &body.parent_cluster,
    &body.pattern,
    body.dry_run,
    body.delete_hsm_group,
  )
  .await
  .map_err(to_handler_error)?;

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

/// `POST /api/v1/hardware-clusters/{target}/configuration` — pin or unpin nodes between clusters by hardware pattern.
#[utoipa::path(post, path = "/hardware-clusters/{target}/configuration", tag = "hardware",
  params(("target" = String, Path, description = "Target cluster name"), SiteHeader),
  request_body = ApplyHwConfigurationRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
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

  service::authorization::validate_user_group_access(
    &infra, &ctx.token, &target,
  )
  .await
  .map_err(to_handler_error)?;
  service::authorization::validate_user_group_access(
    &infra,
    &ctx.token,
    &body.parent_cluster,
  )
  .await
  .map_err(to_handler_error)?;

  let result = crate::service::hw_cluster::apply_hw_configuration(
    &infra,
    &ctx.token,
    crate::service::hw_cluster::ApplyHwConfigurationParams {
      mode: body.mode,
      target_group_name: &target,
      parent_group_name: &body.parent_cluster,
      pattern: &body.pattern,
      dryrun: body.dry_run,
      create_target_group: body.create_target_hsm_group,
      delete_empty_parent_group: body.delete_empty_parent_hsm_group,
    },
  )
  .await
  .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}
