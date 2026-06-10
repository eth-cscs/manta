//! SAT-file HTTP handlers.
//!
//! Per-element endpoints. The CLI's [`apply_sat_file`] plan builder
//! produces a typed sequence of elements; its dispatcher walks the
//! plan and POSTs each element to the section-specific endpoint here.
//!
//! Configuration + session-template entries take one call each:
//!
//! - `POST /api/v1/sat-file/configurations` →
//!   [`post_sat_configuration`] — Body: [`PostSatConfigurationRequest`];
//!   response: a `CfsConfigurationResponse` as JSON.
//! - `POST /api/v1/sat-file/session-templates` →
//!   [`post_sat_session_template`] — Body:
//!   [`PostSatSessionTemplateRequest`]; response:
//!   [`PostSatSessionTemplateResponse`].
//!
//! Image entries are split across three calls so the CLI can monitor
//! the build instead of blocking on one long server round-trip:
//!
//! - `POST /api/v1/sat-file/images/cfs-session` →
//!   [`post_sat_image_cfs_session`] — translate one `images[]` entry
//!   into a CFS session payload and create it. Body:
//!   [`CreateImageCfsSessionRequest`]; response: the freshly-created
//!   [`CfsSessionGetResponse`] (still pending/running).
//! - Monitor via the existing `GET /sessions?name=…` or
//!   `GET /sessions/{name}/logs` (SSE) endpoints — the CLI picks
//!   which based on `--watch-logs`.
//! - `POST /api/v1/sat-file/images/stamp` → [`post_sat_image_stamp`] —
//!   once the session is terminal-complete, the server fetches it,
//!   derives `manta.image_session.{base,groups,configuration}`, and
//!   PATCHes them onto the produced IMS image. Body:
//!   [`StampImageFromSessionRequest`]; response: the patched [`Image`].
//!   Fails fast with 400 when the session produced no `result_id`.
//!
//! The CLI deserialises each response and pretty-prints the assembled
//! four-list summary, so any rename of a field on either side of the
//! wire is user-visible. The wire-format-lock tests at the bottom of
//! this module catch that drift; mirror them when you add a new field.
//!
//! Each handler calls the matching `InfraContext` method on
//! `&infra` directly — the per-trait service shim was removed once the
//! method bodies stopped doing anything beyond plumbing.

use axum::{Json, http::StatusCode, response::IntoResponse};
use manta_backend_dispatcher::interfaces::apply_sat_file::{
  ApplyConfigurationParams as BackendApplyConfigurationParams,
  ApplyImageCreateSessionParams as BackendApplyImageCreateSessionParams,
  ApplyImageStampParams as BackendApplyImageStampParams,
  ApplySessionTemplateParams as BackendApplySessionTemplateParams, SatTrait,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::service::authorization::validate_user_group_vec_access;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, require_k8s_url, require_vault,
  to_handler_error,
};

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/configurations — Apply one SAT configuration entry
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::sat_file::{
  CreateImageCfsSessionRequest, PostSatConfigurationRequest,
  PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
  StampImageFromSessionRequest,
};

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
    .map_err(to_handler_error)?;

  // CFS configurations are not HSM-group-scoped — the SAT
  // `configurations[]` entry only carries name + layers (git URL,
  // branch, playbook), with no group field. Access control here
  // relies on the backend's RBAC layer (CSM/OCHAMI), matching the
  // convention used for other non-group-scoped handlers (see
  // ARCHITECTURE.md "Security model").
  let cfg = infra
    .backend
    .apply_configuration(BackendApplyConfigurationParams {
      shasta_token: &ctx.token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      gitea_base_url: infra.gitea_base_url,
      gitea_token: &gitea_token,
      configuration: body.configuration,
      dry_run: body.dry_run,
      overwrite: body.overwrite,
    })
    .await
    .map_err(to_handler_error)?;

  Ok(Json(cfg))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/images/cfs-session — Create the CFS session that
// will build the image, but do not wait for it or stamp the result. The
// CLI drives the monitor + stamp steps via the existing session endpoints
// and the companion `/sat-file/images/stamp` endpoint below.
// ---------------------------------------------------------------------------

#[utoipa::path(post, path = "/sat-file/images/cfs-session", tag = "sat-file",
  params(SiteHeader),
  request_body = CreateImageCfsSessionRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "CFS session created",         body = serde_json::Value),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured", body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/images/cfs-session` — translate one SAT
/// `images[]` entry into a CFS session payload and create it. Returns
/// the freshly-created [`CfsSessionGetResponse`] so the CLI can drive
/// the monitor + stamp steps itself.
#[tracing::instrument(skip_all)]
pub async fn post_sat_image_cfs_session(
  ctx: RequestCtx,
  Json(body): Json<CreateImageCfsSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_image_cfs_session dry_run={}", body.dry_run);
  let infra = ctx.infra();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let target_groups =
    crate::service::sat_groups::extract_image_groups(&body.image);

  validate_user_group_vec_access(&infra, &ctx.token, &target_groups)
    .await
    .map_err(to_handler_error)?;

  let session = infra
    .backend
    .apply_sat_image_create_session(BackendApplyImageCreateSessionParams {
      shasta_token: &ctx.token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      image: body.image,
      ref_lookup: body.ref_lookup,
      ansible_verbosity: body.ansible_verbosity,
      ansible_passthrough: body.ansible_passthrough.as_deref(),
      dry_run: body.dry_run,
    })
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json::<CfsSessionGetResponse>(session)))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/images/stamp — Given a (terminal-complete) CFS
// session name, fetch it, derive `manta.image_session.{base,groups,
// configuration}` from it, and PATCH them onto the IMS image the session
// produced. Fails fast when the session has no result image.
// ---------------------------------------------------------------------------

#[utoipa::path(post, path = "/sat-file/images/stamp", tag = "sat-file",
  params(SiteHeader),
  request_body = StampImageFromSessionRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Image stamped",               body = serde_json::Value),
    (status = 400, description = "Session not complete / no image", body = ErrorResponse),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/images/stamp` — fetch the named CFS session,
/// derive the provenance stamp, and PATCH the produced IMS image.
///
/// Performs two boundary checks before delegating to the backend:
/// the caller must have access to every HSM group the session
/// targets, and the session must have produced a result image. See
/// [`crate::service::session::validate_session_access`] +
/// [`crate::service::session::require_result_image`].
#[tracing::instrument(skip_all)]
pub async fn post_sat_image_stamp(
  ctx: RequestCtx,
  Json(body): Json<StampImageFromSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_image_stamp cfs_session={}", body.cfs_session_name);
  let infra = ctx.infra();

  let session = crate::service::session::validate_session_access(
    &infra,
    &ctx.token,
    &body.cfs_session_name,
  )
  .await
  .map_err(to_handler_error)?;

  crate::service::session::require_result_image(&session)
    .map_err(to_handler_error)?;

  let image = infra
    .backend
    .apply_sat_image_stamp_from_session(BackendApplyImageStampParams {
      shasta_token: &ctx.token,
      cfs_session_name: &body.cfs_session_name,
    })
    .await
    .map_err(to_handler_error)?;

  Ok(Json::<Image>(image))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/session-templates — Apply one SAT session_template
// ---------------------------------------------------------------------------

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

  let target_groups =
    crate::service::sat_groups::extract_session_template_groups(
      &body.session_template,
    );

  validate_user_group_vec_access(&infra, &ctx.token, &target_groups)
    .await
    .map_err(to_handler_error)?;

  let hsm_group_available_vec = infra
    .backend
    .get_group_name_available(&ctx.token)
    .await
    .map_err(to_handler_error)?;

  let (template, session) = infra
    .backend
    .apply_session_template(BackendApplySessionTemplateParams {
      shasta_token: &ctx.token,
      session_template: body.session_template,
      ref_lookup: body.ref_lookup,
      hsm_group_available_vec: &hsm_group_available_vec,
      reboot: body.reboot,
      dry_run: body.dry_run,
    })
    .await
    .map_err(to_handler_error)?;

  Ok(Json(PostSatSessionTemplateResponse { template, session }))
}

#[cfg(test)]
mod tests {
  //! Locks the JSON wire format of the per-element request/response
  //! types. The CLI builds the request JSON literally and pretty-prints
  //! each response verbatim, so renames or reordering here would break
  //! the wire boundary.

  use super::{
    CreateImageCfsSessionRequest, PostSatConfigurationRequest,
    PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
    StampImageFromSessionRequest,
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

  /// Lock the shape of the CLI's POST /sat-file/images/cfs-session body.
  #[test]
  fn cli_create_image_cfs_session_body_deserialises() {
    let cli_body = serde_json::json!({
      "image": { "name": "img-v1", "ref_name": "base", "configuration": "cfg-v1" },
      "ref_lookup": { "earlier-ref": "abc-123" },
      "ansible_verbosity": 3,
      "ansible_passthrough": "--check",
      "dry_run": false,
    });
    let req: CreateImageCfsSessionRequest =
      serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.image["name"].as_str(), Some("img-v1"));
    assert_eq!(
      req.ref_lookup.get("earlier-ref").map(String::as_str),
      Some("abc-123")
    );
    assert_eq!(req.ansible_verbosity, Some(3));
    assert_eq!(req.ansible_passthrough.as_deref(), Some("--check"));
    assert!(!req.dry_run);
  }

  /// Minimal create-session body — only `image` is required.
  #[test]
  fn cli_create_image_cfs_session_body_with_defaults_deserialises() {
    let cli_body = serde_json::json!({ "image": { "name": "img-v1" } });
    let req: CreateImageCfsSessionRequest =
      serde_json::from_value(cli_body).unwrap();
    assert!(req.ref_lookup.is_empty());
    assert_eq!(req.ansible_verbosity, None);
    assert_eq!(req.ansible_passthrough, None);
    assert!(!req.dry_run);
  }

  /// Lock the shape of the CLI's POST /sat-file/images/stamp body —
  /// just the CFS session name.
  #[test]
  fn cli_stamp_image_body_deserialises() {
    let cli_body = serde_json::json!({ "cfs_session_name": "sat-img-v1" });
    let req: StampImageFromSessionRequest =
      serde_json::from_value(cli_body).unwrap();
    assert_eq!(req.cfs_session_name, "sat-img-v1");
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
