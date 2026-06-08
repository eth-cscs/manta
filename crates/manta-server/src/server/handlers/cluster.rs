//! GET /api/v1/groups/nodes (canonical) and /clusters (deprecated alias).

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/groups/nodes
// ---------------------------------------------------------------------------

/// Query parameters for `GET /groups/nodes`.
#[derive(Deserialize, IntoParams)]
pub struct ClusterQuery {
  /// HSM group name to list nodes for. When omitted the response covers
  /// every group the bearer token can access.
  pub hsm_group: Option<String>,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status: Option<String>,
}

/// GET /groups/nodes — list nodes in a group with optional status filter.
#[utoipa::path(get, path = "/groups/nodes", tag = "groups",
  params(ClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of group nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",         body = ErrorResponse),
    (status = 500, description = "Internal error",       body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_groups_nodes(
  ctx: RequestCtx,
  Query(q): Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::cluster::GetClusterParams {
    group_name: q.hsm_group,
    settings_group_name: None,
    status_filter: q.status,
  };

  let nodes = service::cluster::get_cluster_nodes(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(nodes))
}

/// DEPRECATED alias for `GET /groups/nodes`. Logs a server-side warning,
/// then delegates to the canonical handler. Old path kept for one
/// release.
#[utoipa::path(get, path = "/clusters", tag = "clusters",
  params(ClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "[DEPRECATED] use /groups/nodes — list of group nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                                          body = ErrorResponse),
    (status = 500, description = "Internal error",                                        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_clusters_deprecated(
  ctx: RequestCtx,
  q: Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::warn!(
    "deprecated endpoint: GET /clusters — use /groups/nodes instead"
  );
  get_groups_nodes(ctx, q).await
}
