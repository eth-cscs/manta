//! HSM group CRUD + membership handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/groups
// ---------------------------------------------------------------------------

/// Query parameters for `GET /groups`.
#[derive(Deserialize, IntoParams)]
pub struct GroupQuery {
  /// Exact group name; returns all groups when `None`.
  pub name: Option<String>,
}

/// GET /groups/available — list HSM group names the token can access.
///
/// Backs CLI authorization helpers that used to call
/// `backend.get_group_name_available` directly.
#[utoipa::path(get, path = "/groups/available", tag = "groups",
  params(SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of accessible group names", body = Vec<String>),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_available_groups(
  ctx: RequestCtx,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();
  let names = infra
    .get_group_name_available(&ctx.token)
    .await
    .map_err(to_handler_error)?;
  Ok(Json(names))
}

/// GET /groups — list HSM groups, optionally filtered by name.
#[utoipa::path(get, path = "/groups", tag = "groups",
  params(GroupQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of groups", body = serde_json::Value),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_groups(
  ctx: RequestCtx,
  Query(q): Query<GroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::group::GetGroupParams {
    group_name: q.name,
    settings_hsm_group_name: None,
  };

  let groups = service::group::get_groups(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(groups))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{label}
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /groups/{label}`.
#[derive(Deserialize, IntoParams)]
pub struct DeleteGroupQuery {
  /// Delete even if the group still has members (default: false).
  #[serde(default)]
  pub force: bool,
}

/// DELETE /groups/{label} — remove an HSM group.
#[utoipa::path(delete, path = "/groups/{label}", tag = "groups",
  params(("label" = String, Path, description = "Group label"), DeleteGroupQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Group removed"),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 404, description = "Not found",      body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_group(
  ctx: RequestCtx,
  Path(label): Path<String>,
  Query(q): Query<DeleteGroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_group label={} force={}", label, q.force);
  let infra = ctx.infra();

  service::group::delete_group(&infra, &ctx.token, &label, q.force)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups
// ---------------------------------------------------------------------------

/// POST /groups — create a new HSM group.
#[utoipa::path(post, path = "/groups", tag = "groups",
  params(SiteHeader),
  request_body = manta_backend_dispatcher::types::Group,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Group created",    body = serde_json::Value),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 409, description = "Conflict",         body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn create_group(
  ctx: RequestCtx,
  Json(group): Json<::manta_backend_dispatcher::types::Group>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_group");
  let infra = ctx.infra();

  service::group::create_group(&infra, &ctx.token, group)
    .await
    .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "created": true })),
  ))
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups/{name}/members
// ---------------------------------------------------------------------------

/// Body for `POST /groups/{name}/members`.
#[derive(Deserialize, ToSchema)]
pub struct AddNodesToGroupRequest {
  /// Hostlist expression (xnames, NIDs, or hostlist notation)
  /// identifying the new member set for the group.
  pub hosts_expression: String,
}

/// Response for `POST /groups/{name}/members`.
#[derive(Serialize, ToSchema)]
pub struct AddNodesToGroupResponse {
  /// Xnames that were added to the group as part of this request.
  pub added: Vec<String>,
  /// Xnames that were removed from the group as part of this request.
  pub removed: Vec<String>,
}

/// POST /groups/{name}/members — replace a group's member list from a host expression.
#[utoipa::path(post, path = "/groups/{name}/members", tag = "groups",
  params(("name" = String, Path, description = "Group name"), SiteHeader),
  request_body = AddNodesToGroupRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Members updated",   body = AddNodesToGroupResponse),
    (status = 400, description = "Bad request",       body = ErrorResponse),
    (status = 401, description = "Unauthorized",      body = ErrorResponse),
    (status = 500, description = "Internal error",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_nodes_to_group(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Json(body): Json<AddNodesToGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "add_nodes_to_group group={} hosts={}",
    name,
    body.hosts_expression
  );
  let infra = ctx.infra();

  let (added, removed) = service::group::add_nodes_to_group(
    &infra,
    &ctx.token,
    &name,
    &body.hosts_expression,
  )
  .await
  .map_err(to_handler_error)?;

  Ok(Json(AddNodesToGroupResponse { added, removed }))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{name}/members — Remove nodes from HSM group
// ---------------------------------------------------------------------------

/// Request body for `DELETE /groups/{name}/members`.
#[derive(Deserialize, ToSchema)]
pub struct DeleteGroupMembersRequest {
  /// Hosts expression (xnames, nids, or hostlist notation) identifying nodes to remove.
  pub xnames_expression: String,
  /// When true, validates the request without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/groups/{name}/members` — remove nodes from an HSM group.
#[utoipa::path(delete, path = "/groups/{name}/members", tag = "groups",
  params(("name" = String, Path, description = "Group name"), SiteHeader),
  request_body = DeleteGroupMembersRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Members removed"),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_group_members(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Json(body): Json<DeleteGroupMembersRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "delete_group_members group={} xnames_expression={} dry_run={}",
    name,
    body.xnames_expression,
    body.dry_run
  );
  let infra = ctx.infra();

  service::group::delete_group_members(
    &infra,
    &ctx.token,
    &name,
    &body.xnames_expression,
    body.dry_run,
  )
  .await
  .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}
