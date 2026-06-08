//! Hardware inventory queries.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/groups/hardware (canonical) and /hardware-clusters (deprecated)
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::queries::{
  HardwareClusterQuery, HardwareNodesListQuery,
};

/// GET /groups/hardware — summarize hardware components per node for a group.
#[utoipa::path(get, path = "/groups/hardware", tag = "groups",
  params(HardwareClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Hardware summary for group nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                      body = ErrorResponse),
    (status = 500, description = "Internal error",                    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_groups_hardware(
  ctx: RequestCtx,
  Query(q): Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::hardware::GetHardwareClusterParams {
    group_name: q.hsm_group,
    settings_hsm_group_name: None,
  };

  let result =
    service::hardware::get_hardware_cluster(&infra, &ctx.token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "hsm_group_name": result.hsm_group_name,
    "node_summaries": result.node_summaries,
  })))
}

/// DEPRECATED alias for `GET /groups/hardware`. Logs a server-side
/// warning and delegates to the canonical handler. Old path kept for
/// one release.
#[utoipa::path(get, path = "/hardware-clusters", tag = "hardware",
  params(HardwareClusterQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "[DEPRECATED] use /groups/hardware — hardware summary for group nodes", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                                                          body = ErrorResponse),
    (status = 500, description = "Internal error",                                                        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_hardware_clusters_deprecated(
  ctx: RequestCtx,
  q: Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::warn!(
    "deprecated endpoint: GET /hardware-clusters — use /groups/hardware instead"
  );
  get_groups_hardware(ctx, q).await
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-nodes-list
// ---------------------------------------------------------------------------


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
  let infra = ctx.infra();

  let params = service::hardware::GetHardwareNodesListParams {
    host_expression: q.xnames,
  };

  let result =
    service::hardware::get_hardware_nodes_list(&infra, &ctx.token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "node_summaries": result.node_summaries,
  })))
}

// ===========================================================================
// WRITE ENDPOINTS
// ===========================================================================
