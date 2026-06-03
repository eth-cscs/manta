//! SAT file apply orchestration (backend trait + HSM groups).
//!
//! Rendering (Jinja2), parsing, the `image_only` /
//! `session_template_only` filters, the topological sort of images,
//! and the dispatch loop all run client-side. This layer hosts thin
//! pass-throughs to the four `SatTrait` methods that csm-rs (and any
//! other backend) implements:
//!
//! - [`apply_sat_file`] — legacy whole-file path, still used for SAT
//!   files with a `hardware:` section.
//! - [`apply_configuration`] — one SAT `configurations[]` entry per
//!   call. Fetches the gitea token; the backend reaches into Vault
//!   and the `cray-product-catalog` ConfigMap itself.
//! - [`apply_image`] — one SAT `images[]` entry per call, plus the
//!   CLI's accumulated `ref_lookup` map. Looks up the caller's
//!   available HSM groups (the backend uses them for
//!   `configuration_group_names` access checks).
//! - [`apply_session_template`] — one SAT `session_templates[]` entry
//!   per call, plus `ref_lookup`. Looks up HSM groups (the backend
//!   uses them to enforce `boot_sets.node_groups` access).
//!
//! The backend fetches its own Kubernetes secrets from Vault
//! internally for each call.

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error,
  types::{
    bos::{session::BosSession, session_template::BosSessionTemplate},
    cfs::cfs_configuration_response::CfsConfigurationResponse,
    ims::Image,
  },
};

use crate::server::common::app_context::InfraContext;
pub use manta_shared::shared::params::sat_file::ApplySatFileParams;

/// Apply a pre-rendered SAT file via the backend.
///
/// Returns the four lists of artifacts the backend produced (or would
/// produce, in `dry_run` mode): CFS configurations, IMS images, BOS
/// session templates, and BOS sessions. The handler serialises these as
/// the JSON response body so `manta apply sat-file` can show them.
pub async fn apply_sat_file(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  params: ApplySatFileParams<'_>,
) -> Result<
  (
    Vec<CfsConfigurationResponse>,
    Vec<Image>,
    Vec<BosSessionTemplate>,
    Vec<BosSession>,
  ),
  Error,
> {
  infra
    .apply_sat_file(token, gitea_token, vault_base_url, k8s_api_url, params)
    .await
}

/// Apply a single SAT `configurations[]` entry. Forwards to the backend
/// `SatTrait::apply_configuration` after threading the server-side
/// context (Vault, K8s, Gitea).
#[allow(clippy::too_many_arguments)]
pub async fn apply_configuration(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  configuration: serde_json::Value,
  dry_run: bool,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  infra
    .apply_configuration(
      token,
      gitea_token,
      vault_base_url,
      k8s_api_url,
      configuration,
      dry_run,
      overwrite,
    )
    .await
}

/// Apply a single SAT `images[]` entry. Looks up the caller's
/// available HSM groups (the backend's per-image validation reads them
/// for `configuration_group_names` access checks) and forwards to
/// `SatTrait::apply_image`.
#[allow(clippy::too_many_arguments)]
pub async fn apply_image(
  infra: &InfraContext<'_>,
  token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  image: serde_json::Value,
  ref_lookup: HashMap<String, String>,
  ansible_verbosity: Option<u8>,
  ansible_passthrough: Option<&str>,
  watch_logs: bool,
  timestamps: bool,
  dry_run: bool,
) -> Result<Image, Error> {
  infra
    .apply_image(
      token,
      vault_base_url,
      k8s_api_url,
      image,
      ref_lookup,
      ansible_verbosity,
      ansible_passthrough,
      watch_logs,
      timestamps,
      dry_run,
    )
    .await
}

/// Apply a single SAT `session_templates[]` entry. Looks up the
/// caller's available HSM groups (the backend uses them to enforce
/// `boot_sets.node_groups` access) and forwards to
/// `SatTrait::apply_session_template`.
pub async fn apply_session_template(
  infra: &InfraContext<'_>,
  token: &str,
  session_template: serde_json::Value,
  ref_lookup: HashMap<String, String>,
  reboot: bool,
  dry_run: bool,
) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
  infra
    .apply_session_template(token, session_template, ref_lookup, reboot, dry_run)
    .await
}
