//! IMS image handlers.
//!
//! - `GET    /v2/images` → [`get_images`] —
//!   wraps `service::image::get_images`. Orders oldest-first (newest
//!   last) and filters by name glob and `since`/`until` creation-date
//!   bounds.
//! - `DELETE /v2/images` → [`delete_images`] —
//!   wraps `service::image::delete_images`; with `?dry_run=true`
//!   returns the validation result without deleting.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, parse_iso_datetime, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /v2/images
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::queries::{DeleteImagesQuery, ImageQuery};

/// GET /images — list IMS images, oldest-first (newest last).
#[utoipa::path(get, path = "/images", tag = "images",
  params(ImageQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of images", body = Vec<serde_json::Value>),
    (status = 400, description = "Malformed since/until, or since after until", body = ErrorResponse),
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

  let since = q
    .since
    .as_deref()
    .map(|s| parse_iso_datetime("since", s))
    .transpose()?;
  let until = q
    .until
    .as_deref()
    .map(|s| parse_iso_datetime("until", s))
    .transpose()?;

  let params = service::image::GetImagesParams {
    id: q.id,
    pattern: q.pattern,
    since,
    until,
    limit: q.limit,
  };

  let images = service::image::get_images(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(images))
}

// ---------------------------------------------------------------------------
// DELETE /v2/images — with ?ids=id1,id2&dry_run=true
// ---------------------------------------------------------------------------

/// `DELETE /v2/images` — delete IMS images by ID; validates only when `dry_run=true`.
#[utoipa::path(delete, path = "/images", tag = "images",
  params(DeleteImagesQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
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
