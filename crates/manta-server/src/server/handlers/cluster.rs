//! GET /api/v1/clusters.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/clusters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /clusters`.
#[derive(Deserialize, IntoParams)]
pub struct ClusterQuery {
  /// Cluster (HSM group) name to list nodes for.
  pub hsm_group: Option<String>,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status: Option<String>,
}

/// GET /clusters — list cluster nodes with optional group/status filters.
#[utoipa::path(get, path = "/clusters", tag = "clusters",
  params(ClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of cluster nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",          body = ErrorResponse),
    (status = 500, description = "Internal error",        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_clusters(
  ctx: RequestCtx,
  Query(q): Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::cluster::GetClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
    status_filter: q.status,
  };

  let nodes = service::cluster::get_cluster_nodes(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(nodes))
}
