//! Hardware inventory queries.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-clusters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /hardware-clusters`.
#[derive(Deserialize, IntoParams)]
pub struct HardwareClusterQuery {
  pub hsm_group: Option<String>,
}

/// GET /hardware-clusters — summarize hardware components per node for a cluster.
#[utoipa::path(get, path = "/hardware-clusters", tag = "hardware",
  params(HardwareClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Hardware summary for cluster nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                       body = ErrorResponse),
    (status = 500, description = "Internal error",                     body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_hardware_clusters(
  ctx: RequestCtx,
  Query(q): Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::hardware::GetHardwareClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
  };

  let result = service::hardware::get_hardware_cluster(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "hsm_group_name": result.hsm_group_name,
    "node_summaries": result.node_summaries,
  })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-nodes-list
// ---------------------------------------------------------------------------

/// Query parameters for `GET /hardware-nodes-list`.
#[derive(Deserialize, IntoParams)]
pub struct HardwareNodesListQuery {
  /// Comma-separated xnames.
  pub xnames: String,
}

/// GET /hardware-nodes-list — hardware details for an explicit list of xnames.
#[utoipa::path(get, path = "/hardware-nodes-list", tag = "hardware",
  params(HardwareNodesListQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Hardware details for specified nodes", body = serde_json::Value),
    (status = 400, description = "Bad request",                          body = ErrorResponse),
    (status = 401, description = "Unauthorized",                         body = ErrorResponse),
    (status = 500, description = "Internal error",                       body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_hardware_nodes_list(
  ctx: RequestCtx,
  Query(q): Query<HardwareNodesListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params =
    service::hardware::GetHardwareNodesListParams { xnames: q.xnames };

  let result =
    service::hardware::get_hardware_nodes_list(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "node_summaries": result.node_summaries,
  })))
}

// ===========================================================================
// WRITE ENDPOINTS
// ===========================================================================
