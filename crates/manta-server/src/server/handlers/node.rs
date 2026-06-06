//! Node CRUD handlers.

use axum::{
  Json,
  extract::Path,
  http::StatusCode,
  response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

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

/// Body for `POST /nodes`.
#[derive(Deserialize, ToSchema)]
pub struct AddNodeRequest {
  /// Physical location ID (xname) of the node, e.g. `x3000c0s1b0n0`.
  pub id: String,
  /// Initial HSM group the node belongs to.
  pub group: String,
  /// Whether to register the node as enabled. Defaults to `false`
  /// (disabled) per serde's default for `bool`; CLI's
  /// `manta add node` flips the polarity via `--disabled`.
  #[serde(default)]
  pub enabled: bool,
  /// Optional architecture tag: `"X86"`, `"ARM"`, or `"Other"`.
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
