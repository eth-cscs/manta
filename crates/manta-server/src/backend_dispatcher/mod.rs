//! `StaticBackendDispatcher` trait implementations.
//!
//! Each `impl <Trait> for StaticBackendDispatcher` block just routes the
//! call to the `CSM` or `OCHAMI` variant via the [`dispatch!`] macro
//! defined below; the actual logic lives in `csm-rs` and `ochami-rs`.
//!
//! Without these explicit forwards, calls would fall through to the
//! `SatTrait` (etc.) default "not implemented" impls in the
//! `manta-backend-dispatcher` crate.

use std::collections::HashMap;
use std::pin::Pin;

use chrono::NaiveDateTime;
use futures::AsyncBufRead;
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite};

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_hw_cluster_pin::ApplyHwClusterPin;
use manta_backend_dispatcher::interfaces::apply_sat_file::{
  ApplyConfigurationParams, ApplyImageParams, ApplySatFileParams,
  ApplySessionTemplateParams, SatTrait,
};
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::authentication::AuthenticationTrait;
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::console::ConsoleTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::interfaces::ims::{
  GetImagesAndDetailsTrait, ImsTrait,
};
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;
use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::{
  self, Component, ComponentArrayPostArray, Group, HWInventory,
  HWInventoryByLocationList, HsmActionResponse, K8sDetails, NodeMetadataArray,
  NodeSummary,
};
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_details::LayerDetails;
use manta_backend_dispatcher::types::cfs::cfs_configuration_request::CfsConfigurationRequest;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
  CfsConfigurationResponse, Layer,
};
use manta_backend_dispatcher::types::cfs::component::Component as CfsComponent;
use manta_backend_dispatcher::types::cfs::session::{
  CfsSessionGetResponse, CfsSessionPostRequest,
};
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};
use manta_backend_dispatcher::types::ims::{Image, PatchImage};
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};

use crate::dispatcher::StaticBackendDispatcher;
use StaticBackendDispatcher::*;

/// Dispatches a method call to the underlying backend variant.
///
/// Both `CSM` and `OCHAMI` variants always delegate to the same method
/// with identical arguments, so this macro eliminates the repetitive
/// `match self` boilerplate.
///
/// # Usage
/// ```ignore
/// // async method:
/// dispatch!(self, method_name, arg1, arg2)
/// // sync method:
/// dispatch!(sync self, method_name, arg1)
/// ```
macro_rules! dispatch {
  // async (default): adds `.await` after the call
  ($self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*).await,
      OCHAMI(b) => b.$method($($arg),*).await,
    }
  };
  // sync: no `.await`
  (sync $self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*),
      OCHAMI(b) => b.$method($($arg),*),
    }
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// ApplyHwClusterPin
// ─────────────────────────────────────────────────────────────────────────────

impl ApplyHwClusterPin for StaticBackendDispatcher {
  async fn apply_hw_cluster_pin(
    &self,
    shasta_token: &str,
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
    nodryrun: bool,
    create_target_hsm_group: bool,
    delete_empty_parent_hsm_group: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      apply_hw_cluster_pin,
      shasta_token,
      target_hsm_group_name,
      parent_hsm_group_name,
      pattern,
      nodryrun,
      create_target_hsm_group,
      delete_empty_parent_hsm_group
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ApplySessionTrait
// ─────────────────────────────────────────────────────────────────────────────

impl ApplySessionTrait for StaticBackendDispatcher {
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    hsm_group: Option<&str>,
    repos_name_vec: &[&str],
    repos_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
  ) -> Result<(String, String), Error> {
    dispatch!(
      self,
      apply_session,
      gitea_token,
      gitea_base_url,
      shasta_token,
      cfs_conf_sess_name,
      playbook_yaml_file_name_opt,
      hsm_group,
      repos_name_vec,
      repos_last_commit_id_vec,
      ansible_limit,
      ansible_verbosity,
      ansible_passthrough
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// AuthenticationTrait — wraps the dispatch with structured tracing
// ─────────────────────────────────────────────────────────────────────────────

impl AuthenticationTrait for StaticBackendDispatcher {
  async fn get_api_token(
    &self,
    username: &str,
    password: &str,
  ) -> Result<String, Error> {
    let backend = self.backend_kind();
    tracing::debug!(backend, user = %username, "dispatch: get_api_token");
    let result = dispatch!(self, get_api_token, username, password);
    if let Err(ref e) = result {
      tracing::warn!(
        backend,
        user = %username,
        error = %e,
        "dispatch: get_api_token returned error from backend client"
      );
    }
    result
  }

  async fn validate_api_token(&self, auth_token: &str) -> Result<(), Error> {
    let backend = self.backend_kind();
    tracing::debug!(backend, "dispatch: validate_api_token");
    let result = dispatch!(self, validate_api_token, auth_token);
    if let Err(ref e) = result {
      tracing::warn!(
        backend,
        error = %e,
        "dispatch: validate_api_token returned error from backend client"
      );
    }
    result
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// BootParametersTrait
// ─────────────────────────────────────────────────────────────────────────────

impl BootParametersTrait for StaticBackendDispatcher {
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_all_bootparameters, auth_token)
  }

  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_bootparameters, auth_token, nodes)
  }

  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, add_bootparameters, auth_token, boot_parameters)
  }

  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, update_bootparameters, auth_token, boot_parameters)
  }

  async fn delete_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<String, Error> {
    dispatch!(self, delete_bootparameters, auth_token, boot_parameters)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// CfsTrait
// ─────────────────────────────────────────────────────────────────────────────

impl CfsTrait for StaticBackendDispatcher {
  type T = Pin<Box<dyn AsyncBufRead + Send>>;

  async fn get_session_logs_stream(
    &self,
    shasta_token: &str,
    site_name: &str,
    cfs_session_name: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    dispatch!(
      self,
      get_session_logs_stream,
      shasta_token,
      site_name,
      cfs_session_name,
      timestamps,
      k8s
    )
  }

  async fn get_session_logs_stream_by_xname(
    &self,
    auth_token: &str,
    site_name: &str,
    xname: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    dispatch!(
      self,
      get_session_logs_stream_by_xname,
      auth_token,
      site_name,
      xname,
      timestamps,
      k8s
    )
  }

  async fn post_session(
    &self,
    shasta_token: &str,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    dispatch!(self, post_session, shasta_token, session)
  }

  async fn get_sessions(
    &self,
    auth_token: &str,
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
    dispatch!(
      self,
      get_sessions,
      auth_token,
      session_name_opt,
      limit_opt,
      after_id_opt,
      min_age_opt,
      max_age_opt,
      status_opt,
      name_contains_opt,
      is_succeded_opt,
      tags_opt
    )
  }

  async fn get_and_filter_sessions(
    &self,
    shasta_token: &str,
    hsm_group_name_vec: Vec<String>,
    xname_vec: Vec<&str>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    type_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    dispatch!(
      self,
      get_and_filter_sessions,
      shasta_token,
      hsm_group_name_vec,
      xname_vec,
      min_age_opt,
      max_age_opt,
      type_opt,
      status_opt,
      cfs_session_name_opt,
      limit_number_opt,
      is_succeded_opt
    )
  }

  async fn delete_and_cancel_session(
    &self,
    shasta_token: &str,
    group_available_vec: &[Group],
    cfs_session: &CfsSessionGetResponse,
    cfs_component_vec: &[CfsComponent],
    bss_bootparameter_vec: &[BootParameters],
    dry_run: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete_and_cancel_session,
      shasta_token,
      group_available_vec,
      cfs_session,
      cfs_component_vec,
      bss_bootparameter_vec,
      dry_run
    )
  }

  async fn create_configuration_from_repos(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    repo_name_vec: &[&str],
    local_git_commit_vec: &[&str],
    playbook_file_name_opt: Option<&str>,
  ) -> Result<CfsConfigurationRequest, Error> {
    dispatch!(
      self,
      create_configuration_from_repos,
      gitea_token,
      gitea_base_url,
      repo_name_vec,
      local_git_commit_vec,
      playbook_file_name_opt
    )
  }

  async fn get_configuration(
    &self,
    auth_token: &str,
    cfs_configuration_name_opt: Option<&String>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    dispatch!(self, get_configuration, auth_token, cfs_configuration_name_opt)
  }

  async fn get_and_filter_configuration(
    &self,
    auth_token: &str,
    configuration_name: Option<&str>,
    configuration_name_pattern: Option<&str>,
    hsm_group_name_vec: &[String],
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    dispatch!(
      self,
      get_and_filter_configuration,
      auth_token,
      configuration_name,
      configuration_name_pattern,
      hsm_group_name_vec,
      since_opt,
      until_opt,
      limit_number_opt
    )
  }

  async fn get_configuration_layer_details(
    &self,
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
    site_name: &str,
  ) -> Result<LayerDetails, Error> {
    dispatch!(
      self,
      get_configuration_layer_details,
      gitea_base_url,
      gitea_token,
      layer,
      site_name
    )
  }

  async fn update_runtime_configuration(
    &self,
    auth_token: &str,
    xnames: &[String],
    desired_configuration: &str,
    enabled: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      update_runtime_configuration,
      auth_token,
      xnames,
      desired_configuration,
      enabled
    )
  }

  async fn put_configuration(
    &self,
    shasta_token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
    overwrite: bool,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(
      self,
      put_configuration,
      shasta_token,
      configuration,
      configuration_name,
      overwrite
    )
  }

  async fn get_derivatives(
    &self,
    auth_token: &str,
    configuration_name: &str,
  ) -> Result<
    (
      Option<Vec<CfsSessionGetResponse>>,
      Option<Vec<BosSessionTemplate>>,
      Option<Vec<Image>>,
    ),
    Error,
  > {
    dispatch!(self, get_derivatives, auth_token, configuration_name)
  }

  async fn get_cfs_components(
    &self,
    shasta_token: &str,
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<CfsComponent>, Error> {
    dispatch!(
      self,
      get_cfs_components,
      shasta_token,
      configuration_name,
      components_ids,
      status
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClusterSessionTrait (BOS session)
// ─────────────────────────────────────────────────────────────────────────────

impl ClusterSessionTrait for StaticBackendDispatcher {
  async fn post_template_session(
    &self,
    shasta_token: &str,
    bos_session: types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    dispatch!(self, post_template_session, shasta_token, bos_session)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClusterTemplateTrait (BOS session template)
// ─────────────────────────────────────────────────────────────────────────────

impl ClusterTemplateTrait for StaticBackendDispatcher {
  async fn get_template(
    &self,
    shasta_token: &str,
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_template, shasta_token, bos_session_template_id_opt)
  }

  async fn get_and_filter_templates(
    &self,
    shasta_token: &str,
    hsm_group_name_vec: &[String],
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(
      self,
      get_and_filter_templates,
      shasta_token,
      hsm_group_name_vec,
      hsm_member_vec,
      bos_sessiontemplate_name_opt,
      limit_number_opt
    )
  }

  async fn get_all_templates(
    &self,
    shasta_token: &str,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_all_templates, shasta_token)
  }

  async fn put_template(
    &self,
    shasta_token: &str,
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    dispatch!(self, put_template, shasta_token, bos_template, bos_template_name)
  }

  async fn delete_template(
    &self,
    shasta_token: &str,
    bos_template_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_template, shasta_token, bos_template_id)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentTrait
// ─────────────────────────────────────────────────────────────────────────────

impl ComponentTrait for StaticBackendDispatcher {
  async fn get_all_nodes(
    &self,
    auth_token: &str,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    dispatch!(self, get_all_nodes, auth_token, nid_only)
  }

  async fn get_node_metadata_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Component>, Error> {
    dispatch!(self, get_node_metadata_available, auth_token)
  }

  async fn get(
    &self,
    auth_token: &str,
    id: Option<&str>,
    r#type: Option<&str>,
    state: Option<&str>,
    flag: Option<&str>,
    role: Option<&str>,
    subrole: Option<&str>,
    enabled: Option<&str>,
    software_status: Option<&str>,
    subtype: Option<&str>,
    arch: Option<&str>,
    class: Option<&str>,
    nid: Option<&str>,
    nid_start: Option<&str>,
    nid_end: Option<&str>,
    partition: Option<&str>,
    group: Option<&str>,
    state_only: Option<&str>,
    flag_only: Option<&str>,
    role_only: Option<&str>,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    dispatch!(
      self,
      get,
      auth_token,
      id,
      r#type,
      state,
      flag,
      role,
      subrole,
      enabled,
      software_status,
      subtype,
      arch,
      class,
      nid,
      nid_start,
      nid_end,
      partition,
      group,
      state_only,
      flag_only,
      role_only,
      nid_only
    )
  }

  async fn post_nodes(
    &self,
    auth_token: &str,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    dispatch!(self, post_nodes, auth_token, component)
  }

  async fn delete_node(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_node, auth_token, id)
  }

  async fn nid_to_xname(
    &self,
    auth_token: &str,
    user_input_nid: &str,
    is_regex: bool,
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, nid_to_xname, auth_token, user_input_nid, is_regex)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ConsoleTrait
// ─────────────────────────────────────────────────────────────────────────────

impl ConsoleTrait for StaticBackendDispatcher {
  type T = Box<dyn AsyncWrite + Unpin + Send>;
  type U = Box<dyn AsyncRead + Unpin + Send>;

  async fn attach_to_node_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    xname: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<
    (
      Box<dyn AsyncWrite + Unpin + Send>,
      Box<dyn AsyncRead + Unpin + Send>,
    ),
    Error,
  > {
    dispatch!(
      self,
      attach_to_node_console,
      shasta_token,
      site_name,
      xname,
      width,
      height,
      k8s
    )
  }

  async fn attach_to_session_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    session_name: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<
    (
      Box<dyn AsyncWrite + Unpin + Send>,
      Box<dyn AsyncRead + Unpin + Send>,
    ),
    Error,
  > {
    dispatch!(
      self,
      attach_to_session_console,
      shasta_token,
      site_name,
      session_name,
      width,
      height,
      k8s
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// DeleteConfigurationsAndDataRelatedTrait
// ─────────────────────────────────────────────────────────────────────────────

impl DeleteConfigurationsAndDataRelatedTrait for StaticBackendDispatcher {
  async fn get_data_to_delete(
    &self,
    shasta_token: &str,
    hsm_name_available_vec: &[String],
    configuration_name_pattern_opt: Option<&str>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
  ) -> Result<
    (
      Vec<CfsSessionGetResponse>,
      Vec<(String, String, String)>,
      Vec<String>,
      Vec<String>,
      Vec<(String, String, String)>,
      Vec<CfsConfigurationResponse>,
    ),
    Error,
  > {
    dispatch!(
      self,
      get_data_to_delete,
      shasta_token,
      hsm_name_available_vec,
      configuration_name_pattern_opt,
      since_opt,
      until_opt
    )
  }

  async fn delete(
    &self,
    shasta_token: &str,
    cfs_configuration_name_vec: &[String],
    image_id_vec: &[String],
    cfs_session_name_vec: &[String],
    bos_sessiontemplate_name_vec: &[String],
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete,
      shasta_token,
      cfs_configuration_name_vec,
      image_id_vec,
      cfs_session_name_vec,
      bos_sessiontemplate_name_vec
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// GetImagesAndDetailsTrait
// ─────────────────────────────────────────────────────────────────────────────

impl GetImagesAndDetailsTrait for StaticBackendDispatcher {
  async fn get_images_and_details(
    &self,
    shasta_token: &str,
    hsm_group_name_vec: &[String],
    id_opt: Option<&str>,
    limit_number: Option<&u8>,
  ) -> Result<Vec<(Image, String, String, bool)>, Error> {
    dispatch!(
      self,
      get_images_and_details,
      shasta_token,
      hsm_group_name_vec,
      id_opt,
      limit_number
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// GroupTrait
//
// Method names (and the trait methods they dispatch to) come from the
// external `manta-backend-dispatcher` crate's `GroupTrait`. Can't be
// renamed locally; cleaner names would have to land upstream first.
// ─────────────────────────────────────────────────────────────────────────────

impl GroupTrait for StaticBackendDispatcher {
  async fn get_group_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_group_available, auth_token)
  }

  async fn get_group_name_available(
    &self,
    jwt_token: &str,
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, get_group_name_available, jwt_token)
  }

  async fn add_group(
    &self,
    auth_token: &str,
    hsm_group: Group,
  ) -> Result<Group, Error> {
    dispatch!(self, add_group, auth_token, hsm_group)
  }

  async fn get_member_vec_from_group_name_vec(
    &self,
    auth_token: &str,
    hsm_group_name_vec: &[String],
  ) -> Result<Vec<String>, Error> {
    dispatch!(
      self,
      get_member_vec_from_group_name_vec,
      auth_token,
      hsm_group_name_vec
    )
  }

  async fn get_group_map_and_filter_by_group_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_group_vec,
      auth_token,
      hsm_name_vec
    )
  }

  async fn get_group_map_and_filter_by_member_vec(
    &self,
    auth_token: &str,
    member_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_member_vec,
      auth_token,
      member_vec
    )
  }

  async fn get_all_groups(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_all_groups, auth_token)
  }

  async fn get_group(
    &self,
    auth_token: &str,
    hsm_name: &str,
  ) -> Result<Group, Error> {
    dispatch!(self, get_group, auth_token, hsm_name)
  }

  async fn get_groups(
    &self,
    auth_token: &str,
    hsm_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_groups, auth_token, hsm_name_vec)
  }

  async fn delete_group(
    &self,
    auth_token: &str,
    hsm_group_label: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_group, auth_token, hsm_group_label)
  }

  async fn get_hsm_map_and_filter_by_hsm_name_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_hsm_map_and_filter_by_hsm_name_vec,
      auth_token,
      hsm_name_vec
    )
  }

  async fn post_member(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_member, auth_token, group_label, xname)
  }

  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xnames: &[&str],
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, add_members_to_group, auth_token, group_label, xnames)
  }

  async fn delete_member_from_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_member_from_group, auth_token, group_label, xname)
  }

  async fn migrate_group_members(
    &self,
    auth_token: &str,
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    new_target_hsm_members: &[&str],
    dryrun: bool,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    dispatch!(
      self,
      migrate_group_members,
      auth_token,
      target_hsm_group_name,
      parent_hsm_group_name,
      new_target_hsm_members,
      dryrun
    )
  }

  async fn update_group_members(
    &self,
    auth_token: &str,
    group_name: &str,
    members_to_remove: &[&str],
    members_to_add: &[&str],
  ) -> Result<(), Error> {
    dispatch!(
      self,
      update_group_members,
      auth_token,
      group_name,
      members_to_remove,
      members_to_add
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// HardwareInventory
// ─────────────────────────────────────────────────────────────────────────────

impl HardwareInventory for StaticBackendDispatcher {
  async fn get_inventory_hardware(
    &self,
    auth_token: &str,
    xname: &str,
  ) -> Result<NodeSummary, Error> {
    dispatch!(self, get_inventory_hardware, auth_token, xname)
  }

  async fn get_inventory_hardware_query(
    &self,
    auth_token: &str,
    xname: &str,
    r#type: Option<&str>,
    children: Option<bool>,
    parents: Option<bool>,
    partition: Option<&str>,
    format: Option<&str>,
  ) -> Result<HWInventory, Error> {
    dispatch!(
      self,
      get_inventory_hardware_query,
      auth_token,
      xname,
      r#type,
      children,
      parents,
      partition,
      format
    )
  }

  async fn post_inventory_hardware(
    &self,
    auth_token: &str,
    hardware: HWInventoryByLocationList,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_inventory_hardware, auth_token, hardware)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImsTrait
// ─────────────────────────────────────────────────────────────────────────────

impl ImsTrait for StaticBackendDispatcher {
  async fn get_images(
    &self,
    shasta_token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_images, shasta_token, image_id_opt)
  }

  async fn get_all_images(
    &self,
    shasta_token: &str,
  ) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_all_images, shasta_token)
  }

  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    dispatch!(sync self, filter_images, image_vec)
  }

  async fn update_image(
    &self,
    shasta_token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    dispatch!(self, update_image, shasta_token, image_id, image)
  }

  async fn delete_image(
    &self,
    shasta_token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_image, shasta_token, image_id)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// MigrateBackupTrait
// ─────────────────────────────────────────────────────────────────────────────

impl MigrateBackupTrait for StaticBackendDispatcher {
  async fn migrate_backup(
    &self,
    shasta_token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    dispatch!(self, migrate_backup, shasta_token, bos, destination)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// MigrateRestoreTrait
// ─────────────────────────────────────────────────────────────────────────────

impl MigrateRestoreTrait for StaticBackendDispatcher {
  async fn migrate_restore(
    &self,
    shasta_token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite_group: bool,
    overwrite_configuration: bool,
    overwrite_image: bool,
    overwrite_template: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      migrate_restore,
      shasta_token,
      bos_file,
      cfs_file,
      hsm_file,
      ims_file,
      image_dir,
      overwrite_group,
      overwrite_configuration,
      overwrite_image,
      overwrite_template
    )
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// PCSTrait — power control
//
// `POST /power` returns immediately with a transition id (via
// `pcs_transitions_post`); the CLI then polls `pcs_transitions_get`
// until the transition is `completed`. The older blocking
// `power_*_sync` trait methods (server-side polling loop) have been
// removed.
// ─────────────────────────────────────────────────────────────────────────────

impl PCSTrait for StaticBackendDispatcher {
  async fn pcs_transitions_post(
    &self,
    auth_token: &str,
    operation: &str,
    nodes: &[String],
  ) -> Result<TransitionStartOutput, Error> {
    dispatch!(self, pcs_transitions_post, auth_token, operation, nodes)
  }

  async fn pcs_transitions_get(
    &self,
    auth_token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    dispatch!(self, pcs_transitions_get, auth_token, transition_id)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// RedfishEndpointTrait
// ─────────────────────────────────────────────────────────────────────────────

impl RedfishEndpointTrait for StaticBackendDispatcher {
  async fn get_all_redfish_endpoints(
    &self,
    auth_token: &str,
  ) -> Result<RedfishEndpointArray, Error> {
    dispatch!(self, get_all_redfish_endpoints, auth_token)
  }

  async fn get_redfish_endpoints(
    &self,
    auth_token: &str,
    id: Option<&str>,
    fqdn: Option<&str>,
    r#type: Option<&str>,
    uuid: Option<&str>,
    macaddr: Option<&str>,
    ip_address: Option<&str>,
    last_status: Option<&str>,
  ) -> Result<RedfishEndpointArray, Error> {
    dispatch!(
      self,
      get_redfish_endpoints,
      auth_token,
      id,
      fqdn,
      r#type,
      uuid,
      macaddr,
      ip_address,
      last_status
    )
  }

  async fn add_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpointArray,
  ) -> Result<(), Error> {
    dispatch!(self, add_redfish_endpoint, auth_token, redfish_endpoint)
  }

  async fn update_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpoint,
  ) -> Result<(), Error> {
    dispatch!(self, update_redfish_endpoint, auth_token, redfish_endpoint)
  }

  async fn delete_redfish_endpoint(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    dispatch!(self, delete_redfish_endpoint, auth_token, id)
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// SatTrait
// ─────────────────────────────────────────────────────────────────────────────

impl SatTrait for StaticBackendDispatcher {
  async fn apply_sat_file(
    &self,
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
    dispatch!(self, apply_sat_file, params)
  }

  async fn apply_configuration(
    &self,
    params: ApplyConfigurationParams<'_>,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(self, apply_configuration, params)
  }

  async fn apply_image(
    &self,
    params: ApplyImageParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_image, params)
  }

  async fn apply_session_template(
    &self,
    params: ApplySessionTemplateParams<'_>,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    dispatch!(self, apply_session_template, params)
  }
}
