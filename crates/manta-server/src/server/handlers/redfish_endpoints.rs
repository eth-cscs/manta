//! Redfish-endpoints CRUD handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

// ---------------------------------------------------------------------------
// GET /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// Query parameters for `GET /redfish-endpoints`.
#[derive(Deserialize, IntoParams)]
pub struct RedfishEndpointsQuery {
  /// Exact endpoint ID (BMC xname) filter.
  pub id: Option<String>,
  /// FQDN substring filter.
  pub fqdn: Option<String>,
  /// UUID exact-match filter.
  pub uuid: Option<String>,
  /// MAC-address exact-match filter (colon-separated hex).
  pub macaddr: Option<String>,
  /// IP-address exact-match filter (IPv4 or IPv6).
  pub ipaddress: Option<String>,
}

/// GET /redfish-endpoints — list HSM Redfish endpoints with optional filters.
#[utoipa::path(get, path = "/redfish-endpoints", tag = "redfish-endpoints",
  params(RedfishEndpointsQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of Redfish endpoints", body = serde_json::Value),
    (status = 401, description = "Unauthorized",              body = ErrorResponse),
    (status = 500, description = "Internal error",            body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_redfish_endpoints(
  ctx: RequestCtx,
  Query(q): Query<RedfishEndpointsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = GetRedfishEndpointsParams {
    id: q.id,
    fqdn: q.fqdn,
    uuid: q.uuid,
    macaddr: q.macaddr,
    ipaddress: q.ipaddress,
  };

  let endpoints = infra
    .get_redfish_endpoints(&ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(endpoints))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/redfish-endpoints/{id}
// ---------------------------------------------------------------------------

/// DELETE /redfish-endpoints/{id} — remove a Redfish endpoint from HSM.
#[utoipa::path(delete, path = "/redfish-endpoints/{id}", tag = "redfish-endpoints",
  params(("id" = String, Path, description = "BMC xname"), SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Endpoint removed"),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 404, description = "Not found",      body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_redfish_endpoint(
  ctx: RequestCtx,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_redfish_endpoint id={}", id);
  let infra = ctx.infra();

  infra
    .delete_redfish_endpoint(&ctx.token, &id)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// POST /redfish-endpoints — register a new Redfish endpoint in HSM.
#[utoipa::path(post, path = "/redfish-endpoints", tag = "redfish-endpoints",
  params(SiteHeader),
  request_body = UpdateRedfishEndpointParams,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Endpoint registered",  body = serde_json::Value),
    (status = 401, description = "Unauthorized",          body = ErrorResponse),
    (status = 500, description = "Internal error",        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_redfish_endpoint(
  ctx: RequestCtx,
  Json(params): Json<UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_redfish_endpoint");
  let infra = ctx.infra();

  infra
    .add_redfish_endpoint(&ctx.token, params)
    .await
    .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "created": true })),
  ))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// PUT /redfish-endpoints — update an existing Redfish endpoint's properties.
#[utoipa::path(put, path = "/redfish-endpoints", tag = "redfish-endpoints",
  params(SiteHeader),
  request_body = UpdateRedfishEndpointParams,
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Endpoint updated"),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn update_redfish_endpoint(
  ctx: RequestCtx,
  Json(params): Json<UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_redfish_endpoint");
  let infra = ctx.infra();

  infra
    .update_redfish_endpoint(&ctx.token, params)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}
