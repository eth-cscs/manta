//! GET/DELETE /api/v1/images.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, serialize_or_500, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/images
// ---------------------------------------------------------------------------

/// Query parameters for `GET /images`.
#[derive(Deserialize, IntoParams)]
pub struct ImageQuery {
  /// Exact IMS image ID; returns just that image when set.
  pub id: Option<String>,
  /// Reges to filter images by name
  pub pattern: Option<String>,
  /// Cap on the number of images returned (most recent first).
  pub limit: Option<u8>,
}

/// Wrapper so the image tuple serializes to named fields.
#[derive(Serialize, ToSchema)]
pub struct ImageEntry {
  /// Raw IMS image object (CSM / OpenCHAMI shape, passed through).
  pub image: serde_json::Value,
  /// Name of the CFS configuration linked to this image at build time.
  pub configuration_name: String,
  /// IMS image ID (UUID). Convenience copy of `image.id` for clients
  /// that don't want to parse the inner JSON.
  pub image_id: String,
  /// Whether the image is still linked to its configuration (vs.
  /// configuration was deleted but image survives).
  pub is_linked: bool,
}

/// GET /images — list IMS images with their associated CFS configuration names.
#[utoipa::path(get, path = "/images", tag = "images",
  params(ImageQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of images", body = Vec<ImageEntry>),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_images(
  ctx: RequestCtx,
  Query(q): Query<ImageQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::image::GetImagesParams {
    id: q.id,
    pattern: q.pattern,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let images = service::image::get_images(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(images))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/images — with ?ids=id1,id2&dry_run=true
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /images`.
#[derive(Deserialize, IntoParams)]
pub struct DeleteImagesQuery {
  /// Comma-separated list of IMS image IDs to delete.
  pub ids: String,
  /// When true, validates deletion eligibility without removing anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/images` — delete IMS images by ID; validates only when `dry_run=true`.
#[utoipa::path(delete, path = "/images", tag = "images",
  params(DeleteImagesQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Images deleted or validation result", body = serde_json::Value),
    (status = 400, description = "Bad request",                         body = ErrorResponse),
    (status = 401, description = "Unauthorized",                        body = ErrorResponse),
    (status = 500, description = "Internal error",                      body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_images(
  ctx: RequestCtx,
  Query(q): Query<DeleteImagesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_images ids={} dry_run={}", q.ids, q.dry_run);
  let infra = ctx.infra();

  let id_strings: Vec<String> =
    q.ids.split(',').map(|s| s.trim().to_string()).collect();
  let id_refs: Vec<&str> =
    id_strings.iter().map(std::string::String::as_str).collect();

  if q.dry_run {
    service::image::validate_image_deletion(&infra, &ctx.token, &id_refs, None)
      .await
      .map_err(to_handler_error)?;
    return Ok((
      StatusCode::OK,
      Json(serde_json::json!({ "validated_ids": id_strings })),
    ));
  }

  let deleted =
    service::image::delete_images(&infra, &ctx.token, &id_refs, None)
      .await
      .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({ "deleted": deleted })),
  ))
}
