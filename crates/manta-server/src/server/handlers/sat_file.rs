//! SAT-file HTTP handlers.
//!
//! Three per-element endpoints. The CLI's [`apply_sat_file`] plan
//! builder produces a typed sequence of elements; its dispatcher walks
//! the plan and POSTs each element to the section-specific endpoint
//! here:
//!
//! - `POST /api/v1/sat-file/configurations` →
//!   [`post_sat_configuration`] — one `configurations[]` entry per
//!   call. Body: [`PostSatConfigurationRequest`]; response: a
//!   `CfsConfigurationResponse` as JSON.
//! - `POST /api/v1/sat-file/images` → [`post_sat_image`] — one
//!   `images[]` entry per call, plus the CLI's accumulated
//!   `ref_lookup`. Body: [`PostSatImageRequest`]; response: an
//!   `Image` as JSON.
//! - `POST /api/v1/sat-file/session-templates` →
//!   [`post_sat_session_template`] — one `session_templates[]` entry
//!   per call. Body: [`PostSatSessionTemplateRequest`]; response:
//!   [`PostSatSessionTemplateResponse`].
//!
//! The CLI deserialises each response into `serde_json::Value` and
//! pretty-prints the assembled four-list summary, so any rename of a
//! field on either side of the wire is user-visible. The
//! wire-format-lock tests at the bottom of this module catch that
//! drift; mirror them when you add a new field.
//!
//! Each handler calls the matching `InfraContext` method on
//! `&infra` directly — the per-trait service shim was removed once the
//! method bodies stopped doing anything beyond plumbing.

use axum::{Json, http::StatusCode, response::IntoResponse};
use manta_backend_dispatcher::types::bos::{
  session::BosSession, session_template::BosSessionTemplate,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, display_error, require_k8s_url,
  require_vault,
};

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
    crate::server::common::vault::http_client::get_shasta_vcs_token(
      &ctx.token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(display_error)?;

  let cfg = infra
    .apply_configuration(
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

  let image = infra
    .apply_image(
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

  let (template, session) = infra
    .apply_session_template(
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
  //! Locks the JSON wire format of the per-element request/response
  //! types. The CLI builds the request JSON literally and pretty-prints
  //! each response verbatim, so renames or reordering here would break
  //! the wire boundary.

  use super::{
    PostSatConfigurationRequest, PostSatImageRequest,
    PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
  };

  /// Lock the shape of the CLI's POST /sat-file/configurations body.
  /// Catches renames on either side of the wire.
  #[test]
  fn cli_configuration_body_deserialises() {
    let cli_body = serde_json::json!({
      "configuration": { "name": "cfg-v1", "layers": [] },
      "overwrite": true,
      "dry_run": false,
    });
    let req: PostSatConfigurationRequest =
      serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.configuration["name"].as_str(), Some("cfg-v1"));
    assert!(req.overwrite);
    assert!(!req.dry_run);
  }

  /// Minimal configuration body — only `configuration` is required; the
  /// two booleans default to `false`.
  #[test]
  fn cli_configuration_body_with_defaults_deserialises() {
    let cli_body = serde_json::json!({
      "configuration": { "name": "cfg-v1" },
    });
    let req: PostSatConfigurationRequest =
      serde_json::from_value(cli_body).unwrap();
    assert!(!req.overwrite);
    assert!(!req.dry_run);
  }

  /// Lock the shape of the CLI's POST /sat-file/images body, including
  /// the `ref_lookup` map the CLI accumulates.
  #[test]
  fn cli_image_body_deserialises() {
    let cli_body = serde_json::json!({
      "image": { "name": "img-v1", "ref_name": "base", "configuration": "cfg-v1" },
      "ref_lookup": { "earlier-ref": "abc-123" },
      "ansible_verbosity": 2,
      "ansible_passthrough": "--check",
      "watch_logs": true,
      "timestamps": false,
      "dry_run": false,
    });
    let req: PostSatImageRequest = serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.image["name"].as_str(), Some("img-v1"));
    assert_eq!(
      req.ref_lookup.get("earlier-ref").map(String::as_str),
      Some("abc-123")
    );
    assert_eq!(req.ansible_verbosity, Some(2));
    assert_eq!(req.ansible_passthrough.as_deref(), Some("--check"));
    assert!(req.watch_logs);
    assert!(!req.timestamps);
    assert!(!req.dry_run);
  }

  /// Empty `ref_lookup` is the common case (first image in the plan).
  #[test]
  fn cli_image_body_with_empty_ref_lookup_deserialises() {
    let cli_body = serde_json::json!({
      "image": { "name": "img-v1" },
    });
    let req: PostSatImageRequest = serde_json::from_value(cli_body).unwrap();
    assert!(req.ref_lookup.is_empty());
    assert_eq!(req.ansible_verbosity, None);
    assert!(!req.watch_logs);
    assert!(!req.dry_run);
  }

  /// Lock the shape of the CLI's POST /sat-file/session-templates body.
  #[test]
  fn cli_session_template_body_deserialises() {
    let cli_body = serde_json::json!({
      "session_template": { "name": "st-1", "image": { "image_ref": "base" }, "configuration": "cfg-v1" },
      "ref_lookup": { "base": "image-xyz" },
      "reboot": true,
      "dry_run": false,
    });
    let req: PostSatSessionTemplateRequest =
      serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.session_template["name"].as_str(), Some("st-1"));
    assert_eq!(
      req.ref_lookup.get("base").map(String::as_str),
      Some("image-xyz")
    );
    assert!(req.reboot);
    assert!(!req.dry_run);
  }

  /// Lock the shape of the session_template response body —
  /// `{ template, session? }`. The CLI's dispatcher reads these
  /// two fields by name.
  #[test]
  fn session_template_response_serialises_with_template_and_optional_session() {
    use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

    let body = PostSatSessionTemplateResponse {
      template: BosSessionTemplate {
        name: Some("st-1".to_string()),
        tenant: None,
        description: None,
        enable_cfs: Some(true),
        cfs: None,
        boot_sets: None,
        links: None,
      },
      session: None,
    };
    let v: serde_json::Value = serde_json::to_value(&body).unwrap();
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("template"));
    assert!(obj.contains_key("session"));
    assert_eq!(obj["template"]["name"].as_str(), Some("st-1"));
    assert!(obj["session"].is_null());
  }
}
