//! HSM Redfish-endpoint CRUD handlers.
//!
//! - `GET    /api/v1/redfish-endpoints`        → [`get_redfish_endpoints`]
//! - `POST   /api/v1/redfish-endpoints`        → [`add_redfish_endpoint`]
//! - `PUT    /api/v1/redfish-endpoints`        → [`update_redfish_endpoint`]
//! - `DELETE /api/v1/redfish-endpoints/{id}`   → [`delete_redfish_endpoint`]
//!
//! All wrap `crate::service::redfish::*`.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;
use manta_shared::types::api::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

// ---------------------------------------------------------------------------
// GET /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::queries::RedfishEndpointsQuery;

/// GET /redfish-endpoints — list HSM Redfish endpoints with optional filters.
#[utoipa::path(get, path = "/redfish-endpoints", tag = "redfish-endpoints",
  params(RedfishEndpointsQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    // RedfishEndpointArray lives in manta-backend-dispatcher (third-party,
    // no ToSchema) — kept as Value until upstream derives it.
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

  let endpoints =
    service::redfish::get_redfish_endpoints(&infra, &ctx.token, &params)
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

  service::redfish::delete_redfish_endpoint(&infra, &ctx.token, &id)
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
    (status = 201, description = "Endpoint registered",  body = manta_shared::types::api::responses::CreatedResponse),
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

  service::redfish::add_redfish_endpoint(&infra, &ctx.token, params)
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

  service::redfish::update_redfish_endpoint(&infra, &ctx.token, params)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}
