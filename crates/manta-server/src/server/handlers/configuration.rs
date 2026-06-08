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
  /// Exact configuration name to fetch.
  pub name: Option<String>,
  /// Glob pattern matched against configuration names.
  pub pattern: Option<String>,
  /// HSM group whose associated configurations should be returned.
  pub hsm_group: Option<String>,
  /// Cap on the number of configurations returned (most recent first).
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
  let infra = ctx.infra();

  let params = service::configuration::GetConfigurationParams {
    name: q.name,
    pattern: q.pattern,
    group_name: q.hsm_group,
    settings_hsm_group_name: None,
    since: None,
    until: None,
    limit: q.limit,
  };

  let configs =
    service::configuration::get_configurations(&infra, &ctx.token, &params)
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

  let candidates = service::configuration::get_deletion_candidates(
    &infra,
    &ctx.token,
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
    &ctx.token,
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
