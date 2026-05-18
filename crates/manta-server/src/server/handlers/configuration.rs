//! GET/DELETE /api/v1/configurations.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, parse_iso_datetime, serialize_or_500,
  to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/configurations
// ---------------------------------------------------------------------------

/// Query parameters for `GET /configurations`.
#[derive(Deserialize, IntoParams)]
pub struct ConfigurationQuery {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

/// GET /configurations — list CFS configurations with optional name/pattern/group filters.
#[utoipa::path(get, path = "/configurations", tag = "configurations",
  params(ConfigurationQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of configurations", body = serde_json::Value),
    (status = 401, description = "Unauthorized",           body = ErrorResponse),
    (status = 500, description = "Internal error",         body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_configurations(
  ctx: RequestCtx,
  Query(q): Query<ConfigurationQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::configuration::GetConfigurationParams {
    name: q.name,
    pattern: q.pattern,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    since: None,
    until: None,
    limit: q.limit,
  };

  let configs =
    service::configuration::get_configurations(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(configs))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/configurations — with ?pattern=...&since=...&until=...&dry_run=true
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /configurations`.
#[derive(Deserialize, IntoParams)]
pub struct DeleteConfigurationsQuery {
  /// Glob pattern to match configuration names.
  pub pattern: Option<String>,
  /// ISO-8601 lower bound — only delete configurations created after this date.
  pub since: Option<String>,
  /// ISO-8601 upper bound — only delete configurations created before this date.
  pub until: Option<String>,
  /// When true, returns deletion candidates without removing anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/configurations` — delete CFS configurations and all derived artifacts.
#[utoipa::path(delete, path = "/configurations", tag = "configurations",
  params(DeleteConfigurationsQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Configurations deleted or preview", body = serde_json::Value),
    (status = 400, description = "Bad request",                       body = ErrorResponse),
    (status = 401, description = "Unauthorized",                      body = ErrorResponse),
    (status = 500, description = "Internal error",                    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_configurations(
  ctx: RequestCtx,
  Query(q): Query<DeleteConfigurationsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_configurations dry_run={}", q.dry_run);
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

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

  let candidates = service::configuration::get_deletion_candidates(
    &infra,
    &token,
    None,
    q.pattern.as_deref(),
    since,
    until,
  )
  .await
  .map_err(to_handler_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&candidates)?)));
  }

  service::configuration::delete_configurations_and_derivatives(
    &infra,
    &token,
    &candidates,
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "deleted_configurations": candidates.configuration_names,
      "deleted_images": candidates.image_ids,
    })),
  ))
}

// ===========================================================================
// BATCH A — MEDIUM-COMPLEXITY WRITE ENDPOINTS
// ===========================================================================
