//! Node CRUD handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/nodes
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::queries::NodesQuery;

/// GET /nodes — fetch node details for a given xname expression.
#[utoipa::path(get, path = "/nodes", tag = "nodes",
  params(NodesQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Node details",  body = Vec<manta_shared::types::dto::NodeDetails>),
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
  let infra = ctx.infra();

  let params = service::node::GetNodesParams {
    host_expression: q.xname,
    include_siblings: q.include_siblings.unwrap_or(false),
    status_filter: q.status,
  };

  let nodes = service::node::get_nodes(&infra, &ctx.token, &params)
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
  let infra = ctx.infra();

  service::node::delete_node(&infra, &ctx.token, &id)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/nodes
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::node::AddNodeRequest;

/// POST /nodes — register a new node in HSM and add it to a group.
#[utoipa::path(post, path = "/nodes", tag = "nodes",
  params(SiteHeader),
  request_body = AddNodeRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Node registered",  body = manta_shared::types::wire::responses::AddNodeResponse),
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
  let infra = ctx.infra();

  service::node::add_node(
    &infra,
    &ctx.token,
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
