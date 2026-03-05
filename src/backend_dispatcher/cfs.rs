use std::pin::Pin;

use manta_backend_dispatcher::{
  error::Error,
  interfaces::cfs::CfsTrait,
  types::{
    Group, K8sDetails,
    bos::session_template::BosSessionTemplate,
    bss::BootParameters,
    cfs::{
      cfs_configuration_details::LayerDetails,
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::{CfsConfigurationResponse, Layer},
      component::Component as CfsComponent,
      session::{CfsSessionGetResponse, CfsSessionPostRequest},
    },
    ims::Image,
  },
};

use StaticBackendDispatcher::*;
use chrono::NaiveDateTime;
use futures::AsyncBufRead;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    dispatch!(
      self,
      post_session,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      session
    )
  }

  async fn get_sessions(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
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
      base_url,
      root_cert,
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
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
      shasta_base_url,
      shasta_root_cert,
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
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
      shasta_base_url,
      shasta_root_cert,
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
    shasta_root_cert: &[u8],
    repo_name_vec: &[&str],
    local_git_commit_vec: &[&str],
    playbook_file_name_opt: Option<&str>,
  ) -> Result<CfsConfigurationRequest, Error> {
    dispatch!(
      self,
      create_configuration_from_repos,
      gitea_token,
      gitea_base_url,
      shasta_root_cert,
      repo_name_vec,
      local_git_commit_vec,
      playbook_file_name_opt
    )
  }

  async fn get_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    cfs_configuration_name_opt: Option<&String>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    dispatch!(
      self,
      get_configuration,
      auth_token,
      base_url,
      root_cert,
      cfs_configuration_name_opt
    )
  }

  async fn get_and_filter_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
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
      base_url,
      root_cert,
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
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
    site_name: &str,
  ) -> Result<LayerDetails, Error> {
    dispatch!(
      self,
      get_configuration_layer_details,
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      layer,
      site_name
    )
  }

  async fn update_runtime_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    xnames: &[String],
    desired_configuration: &str,
    enabled: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      update_runtime_configuration,
      auth_token,
      base_url,
      root_cert,
      xnames,
      desired_configuration,
      enabled
    )
  }

  async fn put_configuration(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
    overwrite: bool,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(
      self,
      put_configuration,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration,
      configuration_name,
      overwrite
    )
  }

  // Get all CFS sessions, IMS images and BOS sessiontemplates
  // related to a CFS configuration
  async fn get_derivatives(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    configuration_name: &str,
  ) -> Result<
    (
      Option<Vec<CfsSessionGetResponse>>,
      Option<Vec<BosSessionTemplate>>,
      Option<Vec<Image>>,
    ),
    Error,
  > {
    dispatch!(
      self,
      get_derivatives,
      auth_token,
      base_url,
      root_cert,
      configuration_name
    )
  }

  async fn get_cfs_components(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<
    Vec<manta_backend_dispatcher::types::cfs::component::Component>,
    Error,
  > {
    dispatch!(
      self,
      get_cfs_components,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name,
      components_ids,
      status
    )
  }
}
