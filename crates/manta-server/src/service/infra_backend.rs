//! Backend-dispatcher methods grouped on `InfraContext`.
//!
//! Service code should call `infra.method(token, ...)` instead of
//! reaching into `infra.backend` and re-passing `shasta_base_url` /
//! `shasta_root_cert` at each call site. This file owns the URL/cert
//! unpacking so the rest of the service layer never sees them.

use std::collections::HashMap;

use manta_backend_dispatcher::types::HsmActionResponse;

use chrono::NaiveDateTime;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::authentication::AuthenticationTrait;
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::{Component as HsmComponent, Group};
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::component::Component;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};
use manta_backend_dispatcher::types::ims::{Image, PatchImage};
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};
use manta_backend_dispatcher::types::{
  ComponentArrayPostArray, HWInventoryByLocationList,
};
use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};
use crate::server::common::app_context::InfraContext;

/// Data gathered for deletion review and execution.
#[derive(serde::Serialize)]
pub struct DeletionCandidates {
  /// CFS sessions whose desired-config matches a candidate configuration.
  pub cfs_sessions_to_delete: Vec<CfsSessionGetResponse>,
  /// BOS session templates to delete: `(name, cfs_config, description)`.
  pub bos_sessiontemplate_tuples: Vec<(String, String, String)>,
  /// IMS image IDs to delete (built by the matching sessions).
  pub image_ids: Vec<String>,
  /// Names of the configurations selected for deletion.
  pub configuration_names: Vec<String>,
  /// CFS sessions summary tuples: `(name, config_name, status)`.
  pub cfs_session_tuples: Vec<(String, String, String)>,
  /// Full configuration objects selected for deletion.
  pub configurations: Vec<CfsConfigurationResponse>,
}

impl InfraContext<'_> {
  /// Stable label for the active backend (`csm`, `ochami`, ...).
  pub fn backend_kind(&self) -> &'static str {
    self.backend.backend_kind()
  }

  /// List IMS images, optionally restricted to a single id.
  pub async fn get_images(
    &self,
    token: &str,
    id: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    self.backend.get_images(token, id).await
  }

  /// Delete an IMS image by id.
  pub async fn delete_image(
    &self,
    token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    self.backend.delete_image(token, image_id).await
  }

  /// List the HSM groups the caller's token can access.
  pub async fn get_group_available(
    &self,
    token: &str,
  ) -> Result<Vec<Group>, Error> {
    self.backend.get_group_available(token).await
  }

  /// List BSS boot parameters for all nodes.
  pub async fn get_all_bootparameters(
    &self,
    token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    self.backend.get_all_bootparameters(token).await
  }

  /// Exchange username/password for a CSM bearer token.
  pub async fn get_api_token(
    &self,
    username: &str,
    password: &str,
  ) -> Result<String, Error> {
    self.backend.get_api_token(username, password).await
  }

  /// Verify a CSM bearer token is still accepted by the backend.
  pub async fn validate_api_token(&self, token: &str) -> Result<(), Error> {
    self.backend.validate_api_token(token).await
  }

  /// Resolve a list of HSM group names to their member xnames.
  pub async fn get_member_vec_from_group_name_vec(
    &self,
    token: &str,
    group_names: &[String],
  ) -> Result<Vec<String>, Error> {
    self
      .backend
      .get_member_vec_from_group_name_vec(token, group_names)
      .await
  }

  /// Delete a node by xname.
  pub async fn delete_node(&self, token: &str, id: &str) -> Result<(), Error> {
    self.backend.delete_node(token, id).await.map(|_| ())
  }

  /// Register one or more nodes with HSM.
  pub async fn post_nodes(
    &self,
    token: &str,
    components: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    self.backend.post_nodes(token, components).await
  }

  /// Upload hardware inventory records.
  pub async fn post_inventory_hardware(
    &self,
    token: &str,
    hw_inventory: HWInventoryByLocationList,
  ) -> Result<(), Error> {
    self
      .backend
      .post_inventory_hardware(token, hw_inventory)
      .await
      .map(|_| ())
  }

  /// Add a node to an HSM group.
  pub async fn post_member(
    &self,
    token: &str,
    group: &str,
    id: &str,
  ) -> Result<(), Error> {
    self.backend.post_member(token, group, id).await.map(|_| ())
  }

  /// Fetch a single HSM group by name.
  pub async fn get_group(
    &self,
    token: &str,
    name: &str,
  ) -> Result<Group, Error> {
    self.backend.get_group(token, name).await
  }

  /// Move nodes from `parent` HSM group into `target`.
  pub async fn migrate_group_members(
    &self,
    token: &str,
    target_hsm_name: &str,
    parent_hsm_name: &str,
    xnames: &[&str],
    dry_run: bool,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    self
      .backend
      .migrate_group_members(
        token,
        target_hsm_name,
        parent_hsm_name,
        xnames,
        dry_run,
      )
      .await
  }

  /// List BOS session templates filtered by HSM groups / members.
  pub async fn get_and_filter_templates(
    &self,
    token: &str,
    hsm_group_name_vec: &[String],
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    self
      .backend
      .get_and_filter_templates(
        token,
        hsm_group_name_vec,
        hsm_member_vec,
        bos_sessiontemplate_name_opt,
        limit_number_opt,
      )
      .await
  }

  /// Create a BOS session from a template.
  pub async fn post_template_session(
    &self,
    token: &str,
    bos_session: BosSession,
  ) -> Result<BosSession, Error> {
    self.backend.post_template_session(token, bos_session).await
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

  /// Collect every artefact that would be deleted when removing matching configurations.
  pub async fn get_data_to_delete(
    &self,
    token: &str,
    hsm_name_available_vec: &[String],
    configuration_name_pattern_opt: Option<&str>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
  ) -> Result<DeletionCandidates, Error> {
    let (
      cfs_sessions_to_delete,
      bos_sessiontemplate_tuples,
      image_ids,
      configuration_names,
      cfs_session_tuples,
      configurations,
    ) = self
      .backend
      .get_data_to_delete(
        token,
        hsm_name_available_vec,
        configuration_name_pattern_opt,
        since_opt,
        until_opt,
      )
      .await?;
    Ok(DeletionCandidates {
      cfs_sessions_to_delete,
      bos_sessiontemplate_tuples,
      image_ids,
      configuration_names,
      cfs_session_tuples,
      configurations,
    })
  }

  /// Delete CFS configurations along with their dependent images, sessions, and templates.
  pub async fn delete_configurations_and_dependents(
    &self,
    token: &str,
    cfs_configuration_name_vec: &[String],
    image_id_vec: &[String],
    cfs_session_name_vec: &[String],
    bos_sessiontemplate_name_vec: &[String],
  ) -> Result<(), Error> {
    self
      .backend
      .delete(
        token,
        cfs_configuration_name_vec,
        image_id_vec,
        cfs_session_name_vec,
        bos_sessiontemplate_name_vec,
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
    self
      .backend
      .get_and_filter_sessions(
        token,
        hsm_group_name_vec,
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

  /// Fetch BSS boot parameters for the given xnames.
  pub async fn get_bootparameters(
    &self,
    token: &str,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    self.backend.get_bootparameters(token, xnames).await
  }

  /// Delete BSS boot parameters by host list.
  pub async fn delete_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_bootparameters(token, boot_parameters)
      .await
      .map(|_| ())
  }

  /// Add (create) BSS boot parameters.
  pub async fn add_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .add_bootparameters(token, boot_parameters)
      .await
  }

  /// Update existing BSS boot parameters.
  pub async fn update_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .update_bootparameters(token, boot_parameters)
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

  /// Patch IMS image metadata (link / arch / tags).
  pub async fn update_image(
    &self,
    token: &str,
    image_id: &str,
    patch: &PatchImage,
  ) -> Result<(), Error> {
    self.backend.update_image(token, image_id, patch).await
  }

  /// Filter images in place using the backend's per-site rules.
  pub fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    self.backend.filter_images(image_vec)
  }

  /// List the HSM groups the caller's token can access (names only).
  pub async fn get_group_name_available(
    &self,
    token: &str,
  ) -> Result<Vec<String>, Error> {
    self.backend.get_group_name_available(token).await
  }

  /// Fetch metadata for every HSM node the caller can access.
  pub async fn get_node_metadata_available(
    &self,
    token: &str,
  ) -> Result<Vec<HsmComponent>, Error> {
    self.backend.get_node_metadata_available(token).await
  }

  /// List every HSM group in the system (no access filter).
  pub async fn get_all_groups(&self, token: &str) -> Result<Vec<Group>, Error> {
    self.backend.get_all_groups(token).await
  }

  /// List HSM groups, optionally restricted to a name set.
  pub async fn get_groups(
    &self,
    token: &str,
    hsm_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    self.backend.get_groups(token, hsm_name_vec).await
  }

  /// For each xname, return the HSM groups it belongs to.
  pub async fn get_group_map_and_filter_by_group_vec(
    &self,
    token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    self
      .backend
      .get_group_map_and_filter_by_group_vec(token, hsm_name_vec)
      .await
  }

  /// Delete an HSM group by label.
  pub async fn delete_group(
    &self,
    token: &str,
    label: &str,
  ) -> Result<HsmActionResponse, Error> {
    self.backend.delete_group(token, label).await
  }

  /// Create an HSM group.
  pub async fn add_group(
    &self,
    token: &str,
    group: Group,
  ) -> Result<Group, Error> {
    self.backend.add_group(token, group).await
  }

  /// Remove a single xname from an HSM group.
  pub async fn delete_member_from_group(
    &self,
    token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_member_from_group(token, group_label, xname)
      .await
  }

  /// Add xnames to an HSM group; returns the updated member list.
  pub async fn add_members_to_group(
    &self,
    token: &str,
    group_label: &str,
    members: &[&str],
  ) -> Result<Vec<String>, Error> {
    self
      .backend
      .add_members_to_group(token, group_label, members)
      .await
  }

  /// Replace an HSM group's membership: remove `members_to_remove`, add `members_to_add`.
  pub async fn update_group_members(
    &self,
    token: &str,
    group_name: &str,
    members_to_remove: &[&str],
    members_to_add: &[&str],
  ) -> Result<(), Error> {
    self
      .backend
      .update_group_members(
        token,
        group_name,
        members_to_remove,
        members_to_add,
      )
      .await
  }

  /// Start a PCS power transition (on / off / restart) for the given xnames.
  pub async fn pcs_transitions_post(
    &self,
    token: &str,
    operation: &str,
    nodes: &[String],
  ) -> Result<TransitionStartOutput, Error> {
    self
      .backend
      .pcs_transitions_post(token, operation, nodes)
      .await
  }

  /// Fetch a single PCS power-transition snapshot by id.
  pub async fn pcs_transitions_get(
    &self,
    token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    self.backend.pcs_transitions_get(token, transition_id).await
  }

  /// Fetch Redfish endpoint registrations matching the filters.
  pub async fn get_redfish_endpoints(
    &self,
    token: &str,
    params: &GetRedfishEndpointsParams,
  ) -> Result<RedfishEndpointArray, Error> {
    self
      .backend
      .get_redfish_endpoints(
        token,
        params.id.as_deref(),
        params.fqdn.as_deref(),
        None,
        params.uuid.as_deref(),
        params.macaddr.as_deref(),
        params.ipaddress.as_deref(),
        None,
      )
      .await
  }

  /// Delete a Redfish endpoint registration by id.
  pub async fn delete_redfish_endpoint(
    &self,
    token: &str,
    id: &str,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_redfish_endpoint(token, id)
      .await
      .map(|_| ())
  }

  /// Register a new Redfish endpoint.
  pub async fn add_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> Result<(), Error> {
    let array = RedfishEndpointArray {
      redfish_endpoints: Some(vec![params_to_redfish_endpoint(params)]),
    };
    self.backend.add_redfish_endpoint(token, &array).await
  }

  /// Update an existing Redfish endpoint's properties.
  pub async fn update_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> Result<(), Error> {
    let endpoint = params_to_redfish_endpoint(params);
    self.backend.update_redfish_endpoint(token, &endpoint).await
  }
}

fn params_to_redfish_endpoint(
  params: UpdateRedfishEndpointParams,
) -> RedfishEndpoint {
  RedfishEndpoint {
    id: params.id,
    name: params.name,
    hostname: params.hostname,
    domain: params.domain,
    fqdn: params.fqdn,
    enabled: Some(params.enabled),
    user: params.user,
    password: params.password,
    use_ssdp: Some(params.use_ssdp),
    mac_required: Some(params.mac_required),
    mac_addr: params.mac_addr,
    ip_address: params.ip_address,
    rediscover_on_update: Some(params.rediscover_on_update),
    template_id: params.template_id,
    r#type: None,
    uuid: None,
    discovery_info: None,
  }
}
