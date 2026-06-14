//! Cross-resource analyses:
//! - GET /api/v1/analysis/images — image-centric link graph.
//! - GET /api/v1/analysis/configurations — configuration-deletion safety.

use axum::{Json, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;
use manta_shared::types::api::analysis::BackendSummary;
use manta_shared::types::api::configuration_analysis::ConfigurationAnalysis;

/// GET /analysis/images — image-centric flat projection of every CFS
/// configuration, CFS session, BOS session template, and IMS image
/// visible to the caller. One row per IMS image; see
/// [`BackendSummary`] for column semantics.
#[utoipa::path(get, path = "/analysis/images", tag = "analysis",
  params(SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Image-analysis rows",   body = Vec<BackendSummary>),
    (status = 401, description = "Unauthorized",          body = ErrorResponse),
    (status = 500, description = "Internal error",        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_image_analysis(
  ctx: RequestCtx,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();
  let rows = service::analysis::get_image_analysis(&infra, &ctx.token)
    .await
    .map_err(to_handler_error)?;
  Ok((StatusCode::OK, Json(rows)))
}

/// GET /analysis/configurations — one row per CFS configuration, sorted by
/// `last_updated` ascending, with a `safe_to_delete` verdict derived
/// from cross-resource dependencies (CFS components' `desired_config`
/// and BSS-referenced images' built-with configuration). See
/// [`ConfigurationAnalysis`] for column semantics.
#[utoipa::path(get, path = "/analysis/configurations", tag = "analysis",
  params(SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Configuration analysis rows", body = Vec<ConfigurationAnalysis>),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_configuration_analysis(
  ctx: RequestCtx,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();
  let rows = service::analysis::get_configuration_analysis(&infra, &ctx.token)
    .await
    .map_err(to_handler_error)?;
  Ok((StatusCode::OK, Json(rows)))
}
