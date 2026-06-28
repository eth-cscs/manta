//! SAT-file service wrappers.
//!
//! Thin forwarders from the HTTP handlers to the backend dispatcher,
//! enforcing the CLAUDE.md boundary rule (handlers → service → backend).
//!
//! [`apply_session_template`] and [`validate_sat_file`] also consolidate
//! the duplicate `get_group_name_available` fetch that the handlers
//! previously performed twice (once inside `validate_user_group_vec_access`,
//! once to build the `hsm_group_available_vec` argument): the service
//! function fetches the group list once, validates it in-memory, and
//! forwards it to the backend.

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_sat_file::{
  ApplyConfigurationParams, ApplyImageCreateSessionParams,
  ApplyImageStampParams, ApplySessionTemplateParams, SatTrait,
  ValidateSatFileParams,
};
use manta_backend_dispatcher::types::bos::{
  session::BosSession, session_template::BosSessionTemplate,
};
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization;

/// Apply a single SAT `configurations[]` entry.
///
/// CFS configurations are not HSM-group-scoped so no caller-access
/// check is performed here. The backend's own RBAC layer enforces
/// CSM-level authorization.
#[allow(clippy::too_many_arguments)]
pub async fn apply_configuration(
  infra: &InfraContext<'_>,
  token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  gitea_token: &str,
  configuration: serde_json::Value,
  dry_run: bool,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  infra
    .backend
    .apply_configuration(ApplyConfigurationParams {
      shasta_token: token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      gitea_base_url: infra.gitea_base_url,
      gitea_token,
      configuration,
      dry_run,
      overwrite,
    })
    .await
}

/// Translate one SAT `images[]` entry into a CFS session and create it.
///
/// Returns the created [`CfsSessionGetResponse`] without waiting for the
/// session to complete (the CLI drives monitor + stamp steps itself).
/// Caller-access validation for the image's target groups must be done
/// BEFORE calling this function (see
/// [`crate::service::authorization::validate_user_group_vec_access`]).
#[allow(clippy::too_many_arguments)]
pub async fn create_image_cfs_session(
  infra: &InfraContext<'_>,
  token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  image: serde_json::Value,
  ref_lookup: HashMap<String, String>,
  ansible_verbosity: Option<u8>,
  ansible_passthrough: Option<&str>,
  dry_run: bool,
) -> Result<CfsSessionGetResponse, Error> {
  infra
    .backend
    .apply_sat_image_create_session(ApplyImageCreateSessionParams {
      shasta_token: token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      image,
      ref_lookup,
      ansible_verbosity,
      ansible_passthrough,
      dry_run,
    })
    .await
}

/// Stamp `manta.image_session.*` provenance metadata onto the IMS image
/// produced by a (terminal-complete) CFS session.
///
/// Session-access validation and result-image existence checks must be
/// done BEFORE calling this; see
/// [`crate::service::session::validate_session_access`] and
/// [`crate::service::session::require_result_image`].
pub async fn stamp_image_from_session(
  infra: &InfraContext<'_>,
  token: &str,
  cfs_session_name: &str,
) -> Result<Image, Error> {
  infra
    .backend
    .apply_sat_image_stamp_from_session(ApplyImageStampParams {
      shasta_token: token,
      cfs_session_name,
    })
    .await
}

/// Apply a single SAT `session_templates[]` entry.
///
/// Fetches the caller's accessible group list once; for non-admin callers
/// validates that every group in `target_groups` is accessible, then
/// passes the full list to the backend as `hsm_group_available_vec`.
/// This consolidates the two `get_group_name_available` calls that the
/// handler previously performed (one inside `validate_user_group_vec_access`,
/// one to build the backend param) into a single backend round-trip.
pub async fn apply_session_template(
  infra: &InfraContext<'_>,
  token: &str,
  session_template: serde_json::Value,
  ref_lookup: HashMap<String, String>,
  target_groups: &[String],
  reboot: bool,
  dry_run: bool,
) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
  let hsm_group_available_vec =
    authorization::fetch_group_names_and_validate_access(infra, token, target_groups).await?;
  infra
    .backend
    .apply_session_template(ApplySessionTemplateParams {
      shasta_token: token,
      session_template,
      ref_lookup,
      hsm_group_available_vec: &hsm_group_available_vec,
      reboot,
      dry_run,
    })
    .await
}

/// Pre-flight validate a SAT file against live CSM state without mutating
/// anything.
///
/// Fetches the caller's accessible group list once; for non-admin callers
/// validates that every group in `target_groups` is accessible, then
/// passes the full list to the backend. Same single-fetch consolidation
/// as [`apply_session_template`].
pub async fn validate_sat_file(
  infra: &InfraContext<'_>,
  token: &str,
  sat_file: serde_json::Value,
  target_groups: &[String],
  vault_base_url: &str,
  k8s_api_url: &str,
) -> Result<(), Error> {
  let hsm_group_available_vec =
    authorization::fetch_group_names_and_validate_access(infra, token, target_groups).await?;
  infra
    .backend
    .validate_sat_file(ValidateSatFileParams {
      shasta_token: token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      sat_file,
      hsm_group_available_vec: &hsm_group_available_vec,
    })
    .await
}
