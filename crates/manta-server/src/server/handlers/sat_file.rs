//! POST /api/v1/sat-file.
//!
//! Accepts a pre-rendered SAT YAML plus apply-time flags, forwards them
//! to [`service::sat_file::apply_sat_file`], and returns the four lists
//! of artifacts the backend produced as a [`PostSatFileResponse`]. The
//! CLI deserialises that JSON into `serde_json::Value` and pretty-prints
//! it, so any change to the field names here is user-visible.

use axum::{Json, http::StatusCode, response::IntoResponse};
use manta_backend_dispatcher::types::{
  bos::{session::BosSession, session_template::BosSessionTemplate},
  cfs::cfs_configuration_response::CfsConfigurationResponse,
  ims::Image,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, display_error, require_k8s_url,
  require_vault,
};
use crate::service;

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file — Apply a SAT file
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file`.
///
/// The CLI renders Jinja2, parses the rendered YAML into a structured
/// value, applies the `image_only` / `session_template_only` filters
/// locally, and sends the resulting value in `sat_file`. The server only
/// orchestrates the apply (Vault secrets, HSM groups, backend call).
#[derive(Deserialize, ToSchema)]
pub struct PostSatFileRequest {
  /// Final SAT file as a structured value — Jinja2 already evaluated
  /// and `image_only` / `session_template_only` filters already applied
  /// client-side.
  #[schema(value_type = serde_json::Value)]
  pub sat_file: serde_json::Value,
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
  /// Overwrite existing IMS images or BOS session templates.
  #[serde(default)]
  pub overwrite: bool,
  /// When true, validates the SAT file without creating any resources.
  #[serde(default)]
  pub dry_run: bool,
}

/// Response body for `POST /sat-file`. Each field is the list of objects
/// the backend produced (or would produce, in `dry_run` mode) while
/// realising the SAT file.
#[derive(Serialize, ToSchema)]
pub struct PostSatFileResponse {
  /// CFS configurations created from the SAT file's `configurations`.
  #[schema(value_type = Vec<serde_json::Value>)]
  pub configurations: Vec<CfsConfigurationResponse>,
  /// IMS images built from the SAT file's `images`.
  #[schema(value_type = Vec<serde_json::Value>)]
  pub images: Vec<Image>,
  /// BOS session templates created from `session_templates`.
  #[schema(value_type = Vec<serde_json::Value>)]
  pub session_templates: Vec<BosSessionTemplate>,
  /// BOS sessions triggered when `reboot` was set.
  #[schema(value_type = Vec<serde_json::Value>)]
  pub bos_sessions: Vec<BosSession>,
}

/// `POST /api/v1/sat-file` — apply a pre-rendered SAT file (images, session
/// templates, and CFS sessions).
#[utoipa::path(post, path = "/sat-file", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatFileRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "SAT file applied",               body = PostSatFileResponse),
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
  let infra = ctx.infra();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let gitea_token =
    crate::server::common::vault::http_client::fetch_shasta_vcs_token(
      &ctx.token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(display_error)?;

  let (configurations, images, session_templates, bos_sessions) =
    service::sat_file::apply_sat_file(
      &infra,
      &ctx.token,
      &gitea_token,
      vault_base_url,
      k8s_api_url,
      service::sat_file::ApplySatFileParams {
        sat_file: body.sat_file,
        ansible_verbosity: body.ansible_verbosity,
        ansible_passthrough: body.ansible_passthrough.as_deref(),
        reboot: body.reboot,
        watch_logs: body.watch_logs,
        timestamps: body.timestamps,
        overwrite: body.overwrite,
        dry_run: body.dry_run,
      },
    )
    .await
    .map_err(display_error)?;

  Ok(Json(PostSatFileResponse {
    configurations,
    images,
    session_templates,
    bos_sessions,
  }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/configurations — Apply one SAT configuration entry
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file/configurations` — one entry from the
/// SAT file's `configurations` section, plus per-call flags.
#[derive(Deserialize, ToSchema)]
pub struct PostSatConfigurationRequest {
  /// One SAT `configurations[]` entry as a structured value.
  #[schema(value_type = serde_json::Value)]
  pub configuration: serde_json::Value,
  /// Overwrite an existing CFS configuration of the same name.
  #[serde(default)]
  pub overwrite: bool,
  /// Validate without creating; the response contains a mock
  /// configuration.
  #[serde(default)]
  pub dry_run: bool,
}

#[utoipa::path(post, path = "/sat-file/configurations", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatConfigurationRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Configuration applied",       body = serde_json::Value),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured", body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/configurations` — apply a single SAT
/// configuration entry. Returns the created `CfsConfigurationResponse`.
#[tracing::instrument(skip_all)]
pub async fn post_sat_configuration(
  ctx: RequestCtx,
  Json(body): Json<PostSatConfigurationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_configuration dry_run={}", body.dry_run);
  let infra = ctx.infra();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let gitea_token =
    crate::server::common::vault::http_client::fetch_shasta_vcs_token(
      &ctx.token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(display_error)?;

  let cfg = service::sat_file::apply_configuration(
    &infra,
    &ctx.token,
    &gitea_token,
    vault_base_url,
    k8s_api_url,
    body.configuration,
    body.dry_run,
    body.overwrite,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(cfg))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/images — Apply one SAT image entry
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file/images` — one entry from the SAT
/// file's `images` section, the CLI's accumulated ref_lookup, and
/// per-call flags.
#[derive(Deserialize, ToSchema)]
pub struct PostSatImageRequest {
  /// One SAT `images[]` entry as a structured value.
  #[schema(value_type = serde_json::Value)]
  pub image: serde_json::Value,
  /// `ref_name.or(name) -> image_id` map for previously-created images;
  /// the backend uses it to resolve `base.image_ref`.
  #[serde(default)]
  pub ref_lookup: std::collections::HashMap<String, String>,
  /// Ansible verbosity level (0–4) for the CFS session that builds
  /// the image.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
  /// Stream CFS session logs while the image builds.
  #[serde(default)]
  pub watch_logs: bool,
  /// Prefix streamed log lines with timestamps.
  #[serde(default)]
  pub timestamps: bool,
  /// Validate without creating; the response contains a mock image.
  #[serde(default)]
  pub dry_run: bool,
}

#[utoipa::path(post, path = "/sat-file/images", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatImageRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Image applied",               body = serde_json::Value),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured", body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/images` — apply a single SAT image entry.
/// Returns the created `Image`.
#[tracing::instrument(skip_all)]
pub async fn post_sat_image(
  ctx: RequestCtx,
  Json(body): Json<PostSatImageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_image dry_run={}", body.dry_run);
  let infra = ctx.infra();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let image = service::sat_file::apply_image(
    &infra,
    &ctx.token,
    vault_base_url,
    k8s_api_url,
    body.image,
    body.ref_lookup,
    body.ansible_verbosity,
    body.ansible_passthrough.as_deref(),
    body.watch_logs,
    body.timestamps,
    body.dry_run,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(image))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/session-templates — Apply one SAT session_template
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file/session-templates` — one entry
/// from the SAT file's `session_templates` section, the CLI's
/// accumulated ref_lookup, and per-call flags.
#[derive(Deserialize, ToSchema)]
pub struct PostSatSessionTemplateRequest {
  /// One SAT `session_templates[]` entry as a structured value.
  #[schema(value_type = serde_json::Value)]
  pub session_template: serde_json::Value,
  /// `ref_name.or(name) -> image_id` map for previously-created
  /// images; the backend uses it to resolve `image.image_ref`.
  #[serde(default)]
  pub ref_lookup: std::collections::HashMap<String, String>,
  /// After creating the template, trigger a BOS session to reboot
  /// the targeted nodes through it.
  #[serde(default)]
  pub reboot: bool,
  /// Validate without creating; the response contains a mock
  /// template (and, if `reboot`, no session is returned).
  #[serde(default)]
  pub dry_run: bool,
}

/// Response body for `POST /sat-file/session-templates`. `session`
/// is populated when `reboot` was true and a BOS session was created.
#[derive(Serialize, ToSchema)]
pub struct PostSatSessionTemplateResponse {
  /// The created (or mock, in dry-run) BOS session template.
  #[schema(value_type = serde_json::Value)]
  pub template: BosSessionTemplate,
  /// The BOS session created by the reboot, if any.
  #[schema(value_type = Option<serde_json::Value>)]
  pub session: Option<BosSession>,
}

#[utoipa::path(post, path = "/sat-file/session-templates", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatSessionTemplateRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Session template applied", body = PostSatSessionTemplateResponse),
    (status = 401, description = "Unauthorized",             body = ErrorResponse),
    (status = 500, description = "Internal error",           body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/session-templates` — apply a single SAT
/// session_template entry. Returns the created BOS session template
/// and (if `reboot` was set and we're not in dry-run) the BOS session
/// that was kicked off to reboot the targeted nodes.
#[tracing::instrument(skip_all)]
pub async fn post_sat_session_template(
  ctx: RequestCtx,
  Json(body): Json<PostSatSessionTemplateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_sat_session_template dry_run={} reboot={}",
    body.dry_run,
    body.reboot
  );
  let infra = ctx.infra();

  let (template, session) = service::sat_file::apply_session_template(
    &infra,
    &ctx.token,
    body.session_template,
    body.ref_lookup,
    body.reboot,
    body.dry_run,
  )
  .await
  .map_err(display_error)?;

  Ok(Json(PostSatSessionTemplateResponse { template, session }))
}

#[cfg(test)]
mod tests {
  //! Locks the JSON wire format of `PostSatFileRequest` and
  //! `PostSatFileResponse`. The CLI builds the request JSON literally
  //! and pretty-prints the response value verbatim, so renames or
  //! reordering here would break the wire boundary.

  use super::{PostSatFileRequest, PostSatFileResponse};

  #[test]
  fn empty_response_has_four_named_arrays() {
    let body = PostSatFileResponse {
      configurations: vec![],
      images: vec![],
      session_templates: vec![],
      bos_sessions: vec![],
    };
    let v: serde_json::Value = serde_json::to_value(&body).unwrap();
    let obj = v.as_object().expect("object");
    assert_eq!(obj.len(), 4);
    for key in
      ["configurations", "images", "session_templates", "bos_sessions"]
    {
      assert!(obj.contains_key(key), "missing key: {key}");
      assert!(obj[key].is_array(), "{key} should be an array");
      assert_eq!(obj[key].as_array().unwrap().len(), 0);
    }
  }

  /// Wire-boundary test: a request body shaped exactly the way the CLI
  /// builds it (in `MantaClient::apply_sat_file`) must deserialise into
  /// `PostSatFileRequest`. Catches accidental renames of `sat_file` or
  /// any of the flag fields on either side of the wire.
  #[test]
  fn cli_request_body_deserialises_into_post_sat_file_request() {
    let cli_body = serde_json::json!({
      "sat_file": {
        "configurations": [{ "name": "cfg-v1", "layers": [] }],
        "images": [{ "name": "img-v1", "configuration": "cfg-v1" }],
        "session_templates": [
          { "name": "st-v1", "image": { "image_ref": "img-v1" }, "configuration": "cfg-v1" }
        ]
      },
      "ansible_verbosity": 2,
      "ansible_passthrough": "--check",
      "reboot": true,
      "watch_logs": true,
      "timestamps": false,
      "overwrite": true,
      "dry_run": false,
    });

    let req: PostSatFileRequest = serde_json::from_value(cli_body).unwrap();

    assert_eq!(req.ansible_verbosity, Some(2));
    assert_eq!(req.ansible_passthrough.as_deref(), Some("--check"));
    assert!(req.reboot);
    assert!(req.watch_logs);
    assert!(!req.timestamps);
    assert!(req.overwrite);
    assert!(!req.dry_run);

    // The structured SAT file round-trips intact.
    let sat = req.sat_file.as_object().expect("sat_file is object");
    assert!(sat.contains_key("configurations"));
    assert!(sat.contains_key("images"));
    assert!(sat.contains_key("session_templates"));
    assert_eq!(
      sat["images"][0]["name"].as_str(),
      Some("img-v1"),
      "image name should round-trip"
    );
  }

  /// CLI default-flag form: only required fields and `#[serde(default)]`
  /// fields omitted. Verifies the server accepts the minimal body.
  #[test]
  fn cli_request_body_with_defaults_deserialises() {
    let cli_body = serde_json::json!({
      "sat_file": { "configurations": [], "images": [], "session_templates": [] },
    });
    let req: PostSatFileRequest = serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.ansible_verbosity, None);
    assert_eq!(req.ansible_passthrough, None);
    assert!(!req.reboot);
    assert!(!req.watch_logs);
    assert!(!req.timestamps);
    assert!(!req.overwrite);
    assert!(!req.dry_run);
  }

  /// Missing `sat_file` must fail — there's no default for it.
  #[test]
  fn request_body_without_sat_file_is_rejected() {
    let body = serde_json::json!({ "reboot": true });
    let result = serde_json::from_value::<PostSatFileRequest>(body);
    let err = match result {
      Ok(_) => panic!("expected deserialisation failure"),
      Err(e) => e,
    };
    assert!(
      err.to_string().contains("sat_file"),
      "error should mention the missing field, got: {err}"
    );
  }
}
