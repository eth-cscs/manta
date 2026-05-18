//! Node CRUD handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/nodes
// ---------------------------------------------------------------------------

/// Query parameters for `GET /nodes`.
#[derive(Deserialize, IntoParams)]
pub struct NodesQuery {
  pub xname: String,
  /// Expand results to include nodes sharing the same power supply.
  pub include_siblings: Option<bool>,
  pub status: Option<String>,
}

/// GET /nodes — fetch node details for a given xname expression.
#[utoipa::path(get, path = "/nodes", tag = "nodes",
  params(NodesQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Node details",  body = serde_json::Value),
    (status = 400, description = "Bad request",   body = ErrorResponse),
    (status = 401, description = "Unauthorized",  body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_nodes(
  ctx: RequestCtx,
  Query(q): Query<NodesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::node::GetNodesParams {
    xname: q.xname,
    include_siblings: q.include_siblings.unwrap_or(false),
    status_filter: q.status,
  };

  let nodes = service::node::get_nodes(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(nodes))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/nodes/{id}
// ---------------------------------------------------------------------------

/// DELETE /nodes/{id} — remove a node from HSM by xname or NID.
#[utoipa::path(delete, path = "/nodes/{id}", tag = "nodes",
  params(("id" = String, Path, description = "Node xname or NID"), SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Node removed"),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 404, description = "Not found",    body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_node(
  ctx: RequestCtx,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_node id={}", id);
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::node::delete_node(&infra, &token, &id)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/nodes
// ---------------------------------------------------------------------------

/// Body for `POST /nodes`.
#[derive(Deserialize, ToSchema)]
pub struct AddNodeRequest {
  pub id: String,
  pub group: String,
  #[serde(default)]
  pub enabled: bool,
  pub arch: Option<String>,
}

/// POST /nodes — register a new node in HSM and add it to a group.
#[utoipa::path(post, path = "/nodes", tag = "nodes",
  params(SiteHeader),
  request_body = AddNodeRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Node registered",  body = serde_json::Value),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_node(
  ctx: RequestCtx,
  Json(body): Json<AddNodeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_node id={} group={}", body.id, body.group);
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::node::add_node(
    &infra,
    &token,
    &body.id,
    &body.group,
    body.enabled,
    body.arch,
    None, // hardware_file_path not applicable via HTTP
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "id": body.id })),
  ))
}
