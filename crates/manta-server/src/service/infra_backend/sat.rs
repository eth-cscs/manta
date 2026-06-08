//! SAT-file (apply / per-element) backend methods on `InfraContext`.

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_sat_file::{
  ApplyConfigurationParams as BackendApplyConfigurationParams,
  ApplyImageCreateSessionParams as BackendApplyImageCreateSessionParams,
  ApplyImageStampParams as BackendApplyImageStampParams,
  ApplySatFileParams as BackendApplySatFileParams,
  ApplySessionTemplateParams as BackendApplySessionTemplateParams, SatTrait,
};
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;
use manta_shared::types::params::sat_file::ApplySatFileParams;

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Apply a whole SAT file via the backend (used for files containing a `hardware:` section).
  pub async fn apply_sat_file(
    &self,
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
    let hsm_group_available_vec = self.get_group_name_available(token).await?;
    self
      .backend
      .apply_sat_file(BackendApplySatFileParams {
        shasta_token: token,
        vault_base_url,
        site_name: self.site_name,
        k8s_api_url,
        sat_file: params.sat_file,
        hsm_group_available_vec: &hsm_group_available_vec,
        ansible_verbosity: params.ansible_verbosity,
        ansible_passthrough: params.ansible_passthrough,
        gitea_base_url: self.gitea_base_url,
        gitea_token,
        reboot: params.reboot,
        watch_logs: params.watch_logs,
        timestamps: params.timestamps,
        debug_on_failure: true,
        overwrite: params.overwrite,
        dry_run: params.dry_run,
      })
      .await
  }

  /// Apply a single SAT `configurations[]` entry.
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_configuration(
    &self,
    token: &str,
    gitea_token: &str,
    vault_base_url: &str,
    k8s_api_url: &str,
    configuration: serde_json::Value,
    dry_run: bool,
    overwrite: bool,
  ) -> Result<CfsConfigurationResponse, Error> {
    self
      .backend
      .apply_configuration(BackendApplyConfigurationParams {
        shasta_token: token,
        vault_base_url,
        site_name: self.site_name,
        k8s_api_url,
        gitea_base_url: self.gitea_base_url,
        gitea_token,
        configuration,
        dry_run,
        overwrite,
      })
      .await
  }

  /// Translate one SAT `images[]` entry into a CFS session payload and
  /// create the session. Returns the freshly-created CFS session
  /// resource — its `status.session.status` will be `pending` or
  /// `running`; the caller (typically manta-cli) is responsible for
  /// driving it to completion via the session-status or session-logs
  /// endpoints, then calling [`Self::stamp_image_from_cfs_session`] to
  /// PATCH `manta.image_session.*` onto the produced IMS image.
  ///
  /// This is the first half of the per-image flow; the second half is
  /// [`Self::stamp_image_from_cfs_session`].
  #[allow(clippy::too_many_arguments)]
  pub async fn create_image_cfs_session(
    &self,
    token: &str,
    vault_base_url: &str,
    k8s_api_url: &str,
    image: serde_json::Value,
    ref_lookup: HashMap<String, String>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    dry_run: bool,
  ) -> Result<CfsSessionGetResponse, Error> {
    self
      .backend
      .apply_sat_image_create_session(BackendApplyImageCreateSessionParams {
        shasta_token: token,
        vault_base_url,
        site_name: self.site_name,
        k8s_api_url,
        image,
        ref_lookup,
        ansible_verbosity,
        ansible_passthrough,
        dry_run,
      })
      .await
  }

  /// Fetch the CFS session named `cfs_session_name`, derive
  /// `manta.image_session.{base,groups,configuration}` from it, and
  /// PATCH the resulting IMS image with those keys.
  ///
  /// Fails when the CFS session is not terminal-complete or has no
  /// `result_id` — i.e. produced no image; there is nothing to PATCH
  /// in that case and the backend signals that with an `Error`.
  pub async fn stamp_image_from_cfs_session(
    &self,
    token: &str,
    cfs_session_name: &str,
  ) -> Result<Image, Error> {
    self
      .backend
      .apply_sat_image_stamp_from_session(BackendApplyImageStampParams {
        shasta_token: token,
        cfs_session_name,
      })
      .await
  }

  /// Apply a single SAT `session_templates[]` entry.
  pub async fn apply_session_template(
    &self,
    token: &str,
    session_template: serde_json::Value,
    ref_lookup: HashMap<String, String>,
    reboot: bool,
    dry_run: bool,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    let hsm_group_available_vec = self.get_group_name_available(token).await?;
    self
      .backend
      .apply_session_template(BackendApplySessionTemplateParams {
        shasta_token: token,
        session_template,
        ref_lookup,
        hsm_group_available_vec: &hsm_group_available_vec,
        reboot,
        dry_run,
      })
      .await
  }
}
