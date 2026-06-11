//! HTTP request/response bodies for the per-element SAT-file apply
//! endpoints under `POST /api/v1/sat-file/*`, plus CLI-built params
//! for the backend's whole-file `apply_sat_file` pass-through.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::types::dto::BosSessionTemplate;
use manta_backend_dispatcher::types::bos::session::BosSession;

/// Request body for `POST /api/v1/sat-file/configurations`.
///
/// Carries one entry from the SAT file's `configurations` section
/// plus per-call flags. csm-rs owns the SAT schema; the CLI and server
/// just shuttle the entry through as `serde_json::Value`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

/// Request body for `POST /api/v1/sat-file/images/cfs-session`.
///
/// Carries one entry from the SAT file's `images` section plus the
/// CLI's accumulated `ref_lookup` and the ansible knobs the CFS
/// session needs.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateImageCfsSessionRequest {
  /// One SAT `images[]` entry as a structured value.
  #[schema(value_type = serde_json::Value)]
  pub image: serde_json::Value,
  /// `ref_name.or(name) -> image_id` map for previously-created
  /// images. The backend uses it to resolve `base.image_ref` chains.
  #[serde(default)]
  pub ref_lookup: HashMap<String, String>,
  /// Ansible verbosity level (0–4) for the CFS session that builds
  /// the image.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
  /// Validate without creating; the server returns a mocked complete
  /// session with a `DRYRUN-<uuid>` result id.
  #[serde(default)]
  pub dry_run: bool,
}

/// Request body for `POST /api/v1/sat-file/images/stamp`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StampImageFromSessionRequest {
  /// Name of the (already terminal-complete) CFS session whose result
  /// image should be stamped with `manta.image_session.*` provenance.
  pub cfs_session_name: String,
}

/// Request body for `POST /api/v1/sat-file/session-templates`.
///
/// Carries one entry from the SAT file's `session_templates` section
/// plus the CLI's accumulated `ref_lookup` and per-call flags.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostSatSessionTemplateRequest {
  /// One SAT `session_templates[]` entry as a structured value.
  #[schema(value_type = serde_json::Value)]
  pub session_template: serde_json::Value,
  /// `ref_name.or(name) -> image_id` map for previously-created
  /// images; the backend uses it to resolve `image.image_ref`.
  #[serde(default)]
  pub ref_lookup: HashMap<String, String>,
  /// After creating the template, trigger a BOS session to reboot the
  /// targeted nodes through it.
  #[serde(default)]
  pub reboot: bool,
  /// Validate without creating; the response contains a mock template
  /// and, if `reboot` was set, no session is returned.
  #[serde(default)]
  pub dry_run: bool,
}

/// Response body for `POST /api/v1/sat-file/session-templates`.
///
/// `session` is populated when `reboot` was true and a BOS session
/// was created.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostSatSessionTemplateResponse {
  /// The created (or mock, in dry-run) BOS session template.
  #[schema(value_type = serde_json::Value)]
  pub template: BosSessionTemplate,
  /// The BOS session created by the reboot, if any.
  #[schema(value_type = Option<serde_json::Value>)]
  pub session: Option<BosSession>,
}

/// Parameters for applying a SAT file via the whole-file backend
/// pathway.
///
/// The CLI renders Jinja2, parses the rendered YAML into a structured
/// value, applies the `image_only` / `session_template_only` filters
/// client-side (by removing top-level keys), and forwards the resulting
/// `serde_json::Value` plus the apply-time flags through the server to
/// the backend.
pub struct ApplySatFileParams<'a> {
  /// SAT file parsed into a structured value — Jinja2 already
  /// evaluated and `image_only` / `session_template_only` filters
  /// already applied client-side.
  pub sat_file: serde_json::Value,
  /// Ansible verbosity level (0–4) passed to any CFS sessions
  /// created by this SAT file.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<&'a str>,
  /// When true, reboot affected nodes after the session templates
  /// are applied.
  pub reboot: bool,
  /// When true, stream CFS session logs to the caller as part of
  /// the response.
  pub watch_logs: bool,
  /// When true, prefix each streamed log line with its timestamp.
  pub timestamps: bool,
  /// Overwrite existing CFS configurations or IMS images instead
  /// of erroring on conflict.
  pub overwrite: bool,
  /// Render and validate the SAT file without creating any
  /// resources.
  pub dry_run: bool,
}
