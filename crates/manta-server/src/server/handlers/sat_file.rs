//! POST /api/v1/sat-file.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, display_error, require_k8s_url,
  require_vault, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file — Apply a SAT file
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file`.
#[derive(Deserialize, ToSchema)]
pub struct PostSatFileRequest {
  /// Raw YAML content of the SAT file to apply.
  pub sat_file_content: String,
  /// Inline Jinja2 variable overrides (merged with `values_file_content`).
  pub values: Option<serde_json::Value>,
  /// Raw YAML content of a values file supplying Jinja2 variable overrides.
  pub values_file_content: Option<String>,
  /// Ansible verbosity level passed to any CFS sessions created.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
  /// Reboot nodes after applying the SAT file.
  #[serde(default)]
  pub reboot: bool,
  /// Stream CFS session logs after creation.
  #[serde(default)]
  pub watch_logs: bool,
  /// Prefix log lines with timestamps when streaming logs.
  #[serde(default)]
  pub timestamps: bool,
  /// Only process image sections; skip session templates.
  #[serde(default)]
  pub image_only: bool,
  /// Only process session template sections; skip images.
  #[serde(default)]
  pub session_template_only: bool,
  /// Overwrite existing IMS images or BOS session templates.
  #[serde(default)]
  pub overwrite: bool,
  /// When true, validates the SAT file without creating any resources.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/sat-file` — apply a SAT file (images, session templates, and CFS sessions).
#[utoipa::path(post, path = "/sat-file", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatFileRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "SAT file applied",               body = serde_json::Value),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn post_sat_file(
  ctx: RequestCtx,
  Json(body): Json<PostSatFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_file dry_run={}", body.dry_run);
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let gitea_token =
    crate::server::common::vault::http_client::fetch_shasta_vcs_token(
      &token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(display_error)?;

  service::sat_file::apply_sat_file(
    &infra,
    &token,
    &gitea_token,
    vault_base_url,
    k8s_api_url,
    service::sat_file::ApplySatFileParams {
      sat_file_content: &body.sat_file_content,
      values: body.values.as_ref(),
      values_file_content: body.values_file_content.as_deref(),
      ansible_verbosity: body.ansible_verbosity,
      ansible_passthrough: body.ansible_passthrough.as_deref(),
      reboot: body.reboot,
      watch_logs: body.watch_logs,
      timestamps: body.timestamps,
      image_only: body.image_only,
      session_template_only: body.session_template_only,
      overwrite: body.overwrite,
      dry_run: body.dry_run,
    },
  )
  .await
  .map_err(display_error)?;

  Ok(Json(serde_json::json!({ "applied": true })))
}
