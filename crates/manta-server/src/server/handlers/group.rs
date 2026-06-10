//! HSM group CRUD + membership handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/groups
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::queries::{DeleteGroupQuery, GroupQuery};

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
    .backend
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
    settings_group_name: None,
  };

  let groups = service::group::get_groups(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(groups))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{label}
// ---------------------------------------------------------------------------

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

  // Authorization: caller must have access to the target group.
  service::authorization::validate_user_group_access(
    &infra, &ctx.token, &label,
  )
  .await
  .map_err(to_handler_error)?;

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

  // Authorization: group creation is admin-only. A new label has no
  // existing ownership to validate against, so the only sensible
  // policy without a separate provisioning system is to require the
  // pa_admin role.
  if !crate::server::common::jwt_ops::is_user_admin(&ctx.token) {
    return Err(to_handler_error(
      manta_backend_dispatcher::error::Error::BadRequest(
        "group creation requires admin privileges".to_string(),
      ),
    ));
  }

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

pub use manta_shared::types::wire::group::{
  AddNodesToGroupRequest, AddNodesToGroupResponse, DeleteGroupMembersRequest,
};

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

  // Authorization: caller must have access to the target group.
  service::authorization::validate_user_group_access(&infra, &ctx.token, &name)
    .await
    .map_err(to_handler_error)?;

  let (added, removed) = service::group::add_nodes_to_group(
    &infra,
    &ctx.token,
    &name,
    &body.hosts_expression,
  )
  .await
  .map_err(to_handler_error)?;

  // Emit both `final_members` (canonical) and `removed` (deprecated
  // alias). One release of overlap so existing CLI clients reading
  // `removed` keep working; the next major bump drops `removed`.
  Ok(Json(AddNodesToGroupResponse {
    added,
    final_members: removed.clone(),
    removed,
  }))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{name}/members — Remove nodes from HSM group
// ---------------------------------------------------------------------------

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

  // Authorization: caller must have access to the target group.
  service::authorization::validate_user_group_access(&infra, &ctx.token, &name)
    .await
    .map_err(to_handler_error)?;

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
