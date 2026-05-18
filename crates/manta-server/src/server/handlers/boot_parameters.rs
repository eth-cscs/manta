//! Boot-parameters + apply_boot_config handlers.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, serialize_or_500, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /boot-parameters`.
#[derive(Deserialize, IntoParams)]
pub struct BootParametersQuery {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
}

/// GET /boot-parameters — fetch BSS boot parameters for a group or node list.
#[utoipa::path(get, path = "/boot-parameters", tag = "boot-parameters",
  params(BootParametersQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Boot parameters",  body = serde_json::Value),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_boot_parameters(
  ctx: RequestCtx,
  Query(q): Query<BootParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::boot_parameters::GetBootParametersParams {
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
  };

  let boot_params =
    service::boot_parameters::get_boot_parameters(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(boot_params))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// Body for `DELETE /boot-parameters`.
#[derive(Deserialize, ToSchema)]
pub struct DeleteBootParametersRequest {
  pub hosts: Vec<String>,
}

/// DELETE /boot-parameters — remove BSS boot parameter entries for specified hosts.
#[utoipa::path(delete, path = "/boot-parameters", tag = "boot-parameters",
  params(SiteHeader),
  request_body = DeleteBootParametersRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Boot parameters removed"),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_boot_parameters(
  ctx: RequestCtx,
  Json(body): Json<DeleteBootParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  if body.hosts.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "hosts list must not be empty".to_string(),
      }),
    ));
  }
  tracing::info!("delete_boot_parameters hosts={:?}", body.hosts);
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::delete_boot_parameters(&infra, &token, body.hosts)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// POST /boot-parameters — create a new BSS boot parameters entry.
#[utoipa::path(post, path = "/boot-parameters", tag = "boot-parameters",
  params(SiteHeader),
  request_body = manta_backend_dispatcher::types::bss::BootParameters,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Boot parameters created",  body = serde_json::Value),
    (status = 401, description = "Unauthorized",             body = ErrorResponse),
    (status = 500, description = "Internal error",           body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_boot_parameters(
  ctx: RequestCtx,
  Json(boot_params): Json<
    ::manta_backend_dispatcher::types::bss::BootParameters,
  >,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_boot_parameters");
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::add_boot_parameters(&infra, &token, &boot_params)
    .await
    .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "created": true })),
  ))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// PUT /boot-parameters — update boot image, kernel params, or runtime config for nodes.
#[utoipa::path(put, path = "/boot-parameters", tag = "boot-parameters",
  params(SiteHeader),
  request_body = crate::service::boot_parameters::UpdateBootParametersParams,
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "Boot parameters updated"),
    (status = 400, description = "Bad request",    body = ErrorResponse),
    (status = 401, description = "Unauthorized",   body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn update_boot_parameters(
  ctx: RequestCtx,
  Json(params): Json<service::boot_parameters::UpdateBootParametersParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_boot_parameters");
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::update_boot_parameters(&infra, &token, params)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/boot-config — Apply boot configuration (with ?dry_run=true)
// ---------------------------------------------------------------------------

/// Request body for `POST /boot-config`.
#[derive(Deserialize, ToSchema)]
pub struct ApplyBootConfigRequest {
  /// Node-set expression (xnames, HSM group, or nodeset) identifying the target nodes.
  pub hosts_expression: String,
  /// IMS image ID to set as the boot image.
  pub boot_image_id: Option<String>,
  /// CFS configuration name associated with the boot image.
  pub boot_image_configuration: Option<String>,
  /// Kernel command-line parameters to apply.
  pub kernel_parameters: Option<String>,
  /// CFS configuration to assign as the runtime desired-config.
  pub runtime_configuration: Option<String>,
  /// When true, returns the computed changeset without persisting it.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/boot-config` — apply BSS boot configuration to a set of nodes.
#[utoipa::path(post, path = "/boot-config", tag = "boot-parameters",
  params(SiteHeader),
  request_body = ApplyBootConfigRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Boot config applied or preview", body = serde_json::Value),
    (status = 400, description = "Bad request",                    body = ErrorResponse),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn apply_boot_config(
  ctx: RequestCtx,
  Json(body): Json<ApplyBootConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "apply_boot_config hosts={} dry_run={}",
    body.hosts_expression,
    body.dry_run
  );
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let changeset = service::boot_parameters::prepare_boot_config(
    &infra,
    &token,
    &body.hosts_expression,
    body.boot_image_id.as_deref(),
    body.boot_image_configuration.as_deref(),
    body.kernel_parameters.as_deref(),
  )
  .await
  .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  service::boot_parameters::persist_boot_config(
    &infra,
    &token,
    &changeset,
    body.runtime_configuration.as_deref(),
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "applied": true,
      "nodes": changeset.xname_vec,
      "need_restart": changeset.need_restart,
    })),
  ))
}
