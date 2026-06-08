//! CFS (config + session + components) backend methods on `InfraContext`.

use chrono::NaiveDateTime;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::component::Component;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::{Group, K8sDetails};

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Stream a CFS session's pod logs from the backend.
  ///
  /// Thin wrapper over the backend's `get_session_logs_stream` so the
  /// SSE handler doesn't have to reach into `infra.backend` directly.
  pub async fn get_session_logs_stream(
    &self,
    token: &str,
    session_name: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<impl futures::io::AsyncBufRead + Send + Sized + use<>, Error> {
    crate::service::session::validate_session_access(self, token, session_name)
      .await?;

    self
      .backend
      .get_session_logs_stream(
        token,
        self.site_name,
        session_name,
        timestamps,
        k8s,
      )
      .await
  }

  /// List CFS configurations filtered by name/pattern/HSM groups and date range.
  #[allow(clippy::too_many_arguments)]
  pub async fn get_and_filter_configuration(
    &self,
    token: &str,
    configuration_name: Option<&str>,
    configuration_name_pattern: Option<&str>,
    hsm_group_name_vec: &[String],
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    self
      .backend
      .get_and_filter_configuration(
        token,
        configuration_name,
        configuration_name_pattern,
        hsm_group_name_vec,
        since_opt,
        until_opt,
        limit_number_opt,
      )
      .await
  }

  /// List raw CFS sessions; the filtering args are passed verbatim to the backend.
  #[allow(clippy::too_many_arguments)]
  pub async fn get_sessions(
    &self,
    token: &str,
    session_name_opt: Option<&String>,
    limit_opt: Option<u8>,
    after_id_opt: Option<String>,
    min_age_opt: Option<String>,
    max_age_opt: Option<String>,
    status_opt: Option<String>,
    name_contains_opt: Option<String>,
    is_succeded_opt: Option<bool>,
    tags_opt: Option<String>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    self
      .backend
      .get_sessions(
        token,
        session_name_opt,
        limit_opt,
        after_id_opt,
        min_age_opt,
        max_age_opt,
        status_opt,
        name_contains_opt,
        is_succeded_opt,
        tags_opt,
      )
      .await
  }

  /// List CFS sessions filtered by HSM groups / xnames / age / status / name.
  #[allow(clippy::too_many_arguments)]
  pub async fn get_and_filter_sessions(
    &self,
    token: &str,
    group_name_vec: Vec<String>,
    xname_vec: Vec<&str>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    type_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    self
      .backend
      .get_and_filter_sessions(
        token,
        group_name_vec,
        xname_vec,
        min_age_opt,
        max_age_opt,
        type_opt,
        status_opt,
        cfs_session_name_opt,
        limit_number_opt,
        is_succeded_opt,
      )
      .await
  }

  /// Fetch CFS component records.
  pub async fn get_cfs_components(
    &self,
    token: &str,
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    self
      .backend
      .get_cfs_components(token, configuration_name, components_ids, status)
      .await
  }

  /// Delete a CFS session (and cancel its derived BOS session if still running).
  #[allow(clippy::too_many_arguments)]
  pub async fn delete_and_cancel_session(
    &self,
    token: &str,
    group_available_vec: &[Group],
    cfs_session: &CfsSessionGetResponse,
    cfs_component_vec: &[Component],
    bss_bootparameters_vec: &[BootParameters],
    dry_run: bool,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_and_cancel_session(
        token,
        group_available_vec,
        cfs_session,
        cfs_component_vec,
        bss_bootparameters_vec,
        dry_run,
      )
      .await
  }

  /// Launch a CFS apply-session: build/configure an image or runtime config.
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_session(
    &self,
    gitea_token: &str,
    token: &str,
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    hsm_group: Option<&str>,
    repo_name_vec: &[&str],
    repo_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
  ) -> Result<(String, String), Error> {
    self
      .backend
      .apply_session(
        gitea_token,
        self.gitea_base_url,
        token,
        cfs_conf_sess_name,
        playbook_yaml_file_name_opt,
        hsm_group,
        repo_name_vec,
        repo_last_commit_id_vec,
        ansible_limit,
        ansible_verbosity,
        ansible_passthrough,
      )
      .await
  }

  /// Point the named CFS desired-config at the given xnames.
  pub async fn update_runtime_configuration(
    &self,
    token: &str,
    xnames: &[String],
    new_configuration_name: &str,
    fail_on_missing: bool,
  ) -> Result<(), Error> {
    self
      .backend
      .update_runtime_configuration(
        token,
        xnames,
        new_configuration_name,
        fail_on_missing,
      )
      .await
  }
}
