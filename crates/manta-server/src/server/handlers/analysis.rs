//! Cross-resource analyses.
//!
//! Surfaces the image-centric "link graph" computed by
//! [`crate::service::analysis`] over the four CFS/BOS/IMS resource
//! lists the caller can see. Exposes:
//!
//! - `GET /api/v1/analysis/images` — one row per IMS image, joined
//!   against CFS configurations, CFS sessions, and BOS session
//!   templates. The row carries `safe_to_delete` and orphan hints
//!   the CLI uses for cascading-delete preview.

use axum::{Json, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;
use manta_shared::types::api::analysis::BackendSummary;

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
