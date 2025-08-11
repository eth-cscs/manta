use std::{collections::HashMap, path::PathBuf, pin::Pin};

/// This is the static backend dispatcher
/// To add a new backend:
/// # Add new backend to the StaticBackendDispatcher enum
/// # Add new backend_type to the StaticBackendDispatcher (new) constructor
/// # Add new backend to existing methods in BackendTrait implementation
///
/// To add new functionalities:
/// # Implement new functionalities to BackendTrait implementation
/// NOTE: we assume functionalities are already added to the BackendTrait in 'backend' crate
use manta_backend_dispatcher::{
  contracts::BackendTrait,
  error::Error,
  interfaces::{
    apply_hw_cluster_pin::ApplyHwClusterPin,
    apply_sat_file::SatTrait,
    apply_session::ApplySessionTrait,
    bos::{ClusterSessionTrait, ClusterTemplateTrait},
    bss::BootParametersTrait,
    cfs::CfsTrait,
    commands::CommandsTrait,
    console::ConsoleTrait,
    get_images_and_details::GetImagesAndDetailsTrait,
    hsm::{
      component::ComponentTrait, group::GroupTrait,
      hardware_inventory::HardwareInventory,
      redfish_endpoint::RedfishEndpointTrait,
    },
    ims::ImsTrait,
    migrate_backup::MigrateBackupTrait,
    migrate_restore::MigrateRestoreTrait,
    pcs::PCSTrait,
  },
  types::{
    self,
    bos::session_template::BosSessionTemplate,
    bss::BootParameters,
    cfs::{
      cfs_configuration_details::LayerDetails,
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::{CfsConfigurationResponse, Layer},
      component::Component as CfsComponent,
      session::{CfsSessionGetResponse, CfsSessionPostRequest},
    },
    hsm::inventory::{RedfishEndpoint, RedfishEndpointArray},
    ims::Image,
    Component, ComponentArrayPostArray, Group, HWInventoryByLocationList,
    K8sDetails, NodeMetadataArray,
  },
};

use chrono::NaiveDateTime;
use futures::AsyncBufRead;
use tokio::io::{AsyncRead, AsyncWrite};
use StaticBackendDispatcher::*;

use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;
use serde_json::Value;

#[derive(Clone)]
pub enum StaticBackendDispatcher {
  CSM(Csm),
  OCHAMI(Ochami),
}

impl StaticBackendDispatcher {
  pub fn new(backend_type: &str, base_url: &str, root_cert: &[u8]) -> Self {
    let csm = Csm::new(base_url, root_cert);
    let ochami = Ochami::new(base_url, root_cert);

    match backend_type {
      "csm" => Self::CSM(csm).into(),
      "ochami" => Self::OCHAMI(ochami).into(),
      _ => {
        eprintln!("ERROR - Backend '{}' not supported", backend_type);
        std::process::exit(1);
      }
    }
  }
}

impl GroupTrait for StaticBackendDispatcher {
  async fn get_group_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    match self {
      CSM(b) => b.get_group_available(auth_token).await,
      OCHAMI(b) => b.get_group_available(auth_token).await,
    }
  }

  async fn get_group_name_available(
    &self,
    jwt_token: &str,
  ) -> Result<Vec<String>, Error> {
    match self {
      CSM(b) => b.get_group_name_available(jwt_token).await,
      OCHAMI(b) => b.get_group_name_available(jwt_token).await,
    }
  }

  async fn add_group(
    &self,
    auth_token: &str,
    hsm_group: Group,
  ) -> Result<Group, Error> {
    match self {
      CSM(b) => b.add_group(auth_token, hsm_group).await,
      OCHAMI(b) => b.add_group(auth_token, hsm_group).await,
    }
  }

  // FIXME: rename function to 'get_hsm_group_members'
  async fn get_member_vec_from_group_name_vec(
    &self,
    auth_token: &str,
    hsm_group_name_vec: Vec<String>,
  ) -> Result<Vec<String>, Error> {
    match self {
      CSM(b) => {
        b.get_member_vec_from_group_name_vec(auth_token, hsm_group_name_vec)
          .await
      }
      OCHAMI(b) => {
        b.get_member_vec_from_group_name_vec(auth_token, hsm_group_name_vec)
          .await
      }
    }
  }

  async fn get_group_map_and_filter_by_group_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: Vec<&str>,
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    match self {
      CSM(b) => {
        b.get_group_map_and_filter_by_group_vec(auth_token, hsm_name_vec)
          .await
      }
      OCHAMI(b) => {
        b.get_group_map_and_filter_by_group_vec(auth_token, hsm_name_vec)
          .await
      }
    }
  }

  async fn get_group_map_and_filter_by_member_vec(
    &self,
    auth_token: &str,
    member_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    match self {
      CSM(b) => {
        b.get_group_map_and_filter_by_member_vec(auth_token, member_vec)
          .await
      }
      OCHAMI(b) => {
        b.get_group_map_and_filter_by_member_vec(auth_token, member_vec)
          .await
      }
    }
  }

  async fn get_all_groups(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    match self {
      CSM(b) => b.get_all_groups(auth_token).await,
      OCHAMI(b) => b.get_all_groups(auth_token).await,
    }
  }

  async fn get_group(
    &self,
    auth_token: &str,
    hsm_name: &str,
  ) -> Result<Group, Error> {
    match self {
      CSM(b) => b.get_group(auth_token, hsm_name).await,
      OCHAMI(b) => b.get_group(auth_token, hsm_name).await,
    }
  }

  async fn get_groups(
    &self,
    auth_token: &str,
    hsm_name_vec: Option<&[&str]>,
  ) -> Result<Vec<Group>, Error> {
    match self {
      CSM(b) => b.get_groups(auth_token, hsm_name_vec).await,
      OCHAMI(b) => b.get_groups(auth_token, hsm_name_vec).await,
    }
  }

  async fn delete_group(
    &self,
    auth_token: &str,
    hsm_group_label: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.delete_group(auth_token, hsm_group_label).await,
      OCHAMI(b) => b.delete_group(auth_token, hsm_group_label).await,
    }
  }

  async fn get_hsm_map_and_filter_by_hsm_name_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: Vec<&str>,
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    match self {
      CSM(b) => {
        b.get_hsm_map_and_filter_by_hsm_name_vec(auth_token, hsm_name_vec)
          .await
      }
      OCHAMI(b) => {
        b.get_hsm_map_and_filter_by_hsm_name_vec(auth_token, hsm_name_vec)
          .await
      }
    }
  }

  async fn post_member(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.post_member(auth_token, group_label, xname).await,
      OCHAMI(b) => b.post_member(auth_token, group_label, xname).await,
    }
  }

  // Add members to group.
  // Returns the final list of members in the group.
  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xnames: Vec<&str>,
  ) -> Result<Vec<String>, Error> {
    match self {
      CSM(b) => {
        b.add_members_to_group(auth_token, group_label, xnames)
          .await
      }
      OCHAMI(b) => {
        b.add_members_to_group(auth_token, group_label, xnames)
          .await
      }
    }
  }

  async fn delete_member_from_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.delete_member_from_group(auth_token, group_label, xname)
          .await
      }
      OCHAMI(b) => {
        b.delete_member_from_group(auth_token, group_label, xname)
          .await
      }
    }
  }

  // HSM/GROUP
  async fn migrate_group_members(
    &self,
    auth_token: &str,
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    new_target_hsm_members: Vec<&str>,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    match self {
      CSM(b) => {
        b.migrate_group_members(
          auth_token,
          target_hsm_group_name,
          parent_hsm_group_name,
          new_target_hsm_members,
        )
        .await
      }
      OCHAMI(b) => {
        b.migrate_group_members(
          auth_token,
          target_hsm_group_name,
          parent_hsm_group_name,
          new_target_hsm_members,
        )
        .await
      }
    }
  }

  // HSM/GROUP
  async fn update_group_members(
    &self,
    auth_token: &str,
    group_name: &str,
    members_to_remove: &Vec<String>,
    members_to_add: &Vec<String>,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.update_group_members(
          auth_token,
          group_name,
          members_to_remove,
          members_to_add,
        )
        .await
      }
      OCHAMI(b) => {
        b.update_group_members(
          auth_token,
          group_name,
          members_to_remove,
          members_to_add,
        )
        .await
      }
    }
  }
}

impl HardwareInventory for StaticBackendDispatcher {
  async fn get_inventory_hardware(
    &self,
    auth_token: &str,
    xname: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.get_inventory_hardware(auth_token, xname).await,
      OCHAMI(b) => b.get_inventory_hardware(auth_token, xname).await,
    }
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
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => {
        b.get_inventory_hardware_query(
          auth_token, xname, r#type, children, parents, partition, format,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_inventory_hardware_query(
          auth_token, xname, r#type, children, parents, partition, format,
        )
        .await
      }
    }
  }

  async fn post_inventory_hardware(
    &self,
    auth_token: &str,
    hardware: HWInventoryByLocationList,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.post_inventory_hardware(auth_token, hardware).await,
      OCHAMI(b) => b.post_inventory_hardware(auth_token, hardware).await,
    }
  }
}

impl ComponentTrait for StaticBackendDispatcher {
  async fn get_all_nodes(
    &self,
    auth_token: &str,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    match self {
      CSM(b) => b.get_all_nodes(auth_token, nid_only).await,
      OCHAMI(b) => b.get_all_nodes(auth_token, nid_only).await,
    }
  }

  async fn get_node_metadata_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Component>, Error> {
    match self {
      CSM(b) => b.get_node_metadata_available(auth_token).await,
      OCHAMI(b) => b.get_node_metadata_available(auth_token).await,
    }
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
    match self {
      CSM(b) => {
        b.get(
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
          nid_only,
        )
        .await
      }
      OCHAMI(b) => {
        b.get(
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
          nid_only,
        )
        .await
      }
    }
  }

  async fn post_nodes(
    &self,
    auth_token: &str,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.post_nodes(auth_token, component).await,
      OCHAMI(b) => b.post_nodes(auth_token, component).await,
    }
  }

  async fn delete_node(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.delete_node(auth_token, id).await,
      OCHAMI(b) => b.delete_node(auth_token, id).await,
    }
  }
}

impl PCSTrait for StaticBackendDispatcher {
  async fn power_on_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.power_on_sync(auth_token, nodes).await,
      OCHAMI(b) => b.power_on_sync(auth_token, nodes).await,
    }
  }

  async fn power_off_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.power_off_sync(auth_token, nodes, force).await,
      OCHAMI(b) => b.power_off_sync(auth_token, nodes, force).await,
    }
  }

  async fn power_reset_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.power_reset_sync(auth_token, nodes, force).await,
      OCHAMI(b) => b.power_reset_sync(auth_token, nodes, force).await,
    }
  }
}

impl BootParametersTrait for StaticBackendDispatcher {
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    match self {
      CSM(b) => b.get_all_bootparameters(auth_token).await,
      OCHAMI(b) => b.get_all_bootparameters(auth_token).await,
    }
  }

  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    match self {
      CSM(b) => b.get_bootparameters(auth_token, nodes).await,
      OCHAMI(b) => b.get_bootparameters(auth_token, nodes).await,
    }
  }

  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.add_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.add_bootparameters(auth_token, boot_parameters).await,
    }
  }

  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.update_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.update_bootparameters(auth_token, boot_parameters).await,
    }
  }

  async fn delete_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<String, Error> {
    match self {
      CSM(b) => b.delete_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.delete_bootparameters(auth_token, boot_parameters).await,
    }
  }
}

impl RedfishEndpointTrait for StaticBackendDispatcher {
  async fn get_all_redfish_endpoints(
    &self,
    auth_token: &str,
  ) -> Result<RedfishEndpointArray, Error> {
    match self {
      CSM(b) => b.get_all_redfish_endpoints(auth_token).await,
      OCHAMI(b) => b.get_all_redfish_endpoints(auth_token).await,
    }
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
    match self {
      CSM(b) => {
        b.get_redfish_endpoints(
          auth_token,
          id,
          fqdn,
          r#type,
          uuid,
          macaddr,
          ip_address,
          last_status,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_redfish_endpoints(
          auth_token,
          id,
          fqdn,
          r#type,
          uuid,
          macaddr,
          ip_address,
          last_status,
        )
        .await
      }
    }
  }

  async fn add_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpointArray,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.add_redfish_endpoint(auth_token, redfish_endpoint).await,
      OCHAMI(b) => b.add_redfish_endpoint(auth_token, redfish_endpoint).await,
    }
  }

  async fn update_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpoint,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.update_redfish_endpoint(auth_token, redfish_endpoint)
          .await
      }
      OCHAMI(b) => {
        b.update_redfish_endpoint(auth_token, redfish_endpoint)
          .await
      }
    }
  }

  async fn delete_redfish_endpoint(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.delete_redfish_endpoint(auth_token, id).await,
      OCHAMI(b) => b.delete_redfish_endpoint(auth_token, id).await,
    }
  }
}

impl BackendTrait for StaticBackendDispatcher {
  fn test_backend_trait(&self) -> String {
    println!("in manta backend");
    "in manta backend".to_string()
  }

  // AUTHENTICATION
  async fn get_api_token(&self, site_name: &str) -> Result<String, Error> {
    match self {
      CSM(b) => b.get_api_token(site_name).await,
      OCHAMI(b) => b.get_api_token(site_name).await,
    }
  }

  async fn nid_to_xname(
    &self,
    auth_token: &str,
    user_input_nid: &str,
    is_regex: bool,
  ) -> Result<Vec<String>, Error> {
    match self {
      CSM(b) => b.nid_to_xname(auth_token, user_input_nid, is_regex).await,
      OCHAMI(b) => b.nid_to_xname(auth_token, user_input_nid, is_regex).await,
    }
  }
}

impl CfsTrait for StaticBackendDispatcher {
  type T = Pin<Box<dyn AsyncBufRead + Send>>;

  async fn get_session_logs_stream(
    &self,
    shasta_token: &str,
    site_name: &str,
    cfs_session_name: &str,
    // k8s_api_url: &str,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    match self {
      CSM(b) => {
        b.get_session_logs_stream(
          shasta_token,
          site_name,
          cfs_session_name,
          // k8s_api_url,
          k8s,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_session_logs_stream(
          shasta_token,
          site_name,
          cfs_session_name,
          // k8s_api_url,
          k8s,
        )
        .await
      }
    }
  }

  async fn get_session_logs_stream_by_xname(
    &self,
    auth_token: &str,
    site_name: &str,
    xname: &str,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    match self {
      CSM(b) => {
        b.get_session_logs_stream_by_xname(auth_token, site_name, xname, k8s)
          .await
      }
      OCHAMI(b) => {
        b.get_session_logs_stream_by_xname(auth_token, site_name, xname, k8s)
          .await
      }
    }
  }

  async fn post_session(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    match self {
      CSM(b) => {
        b.post_session(shasta_token, shasta_base_url, shasta_root_cert, session)
          .await
      }
      OCHAMI(b) => {
        b.post_session(shasta_token, shasta_base_url, shasta_root_cert, session)
          .await
      }
    }
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
    match self {
      CSM(b) => {
        b.get_sessions(
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
          tags_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_sessions(
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
          tags_opt,
        )
        .await
      }
    }
  }

  async fn get_and_filter_sessions(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec_opt: Option<Vec<String>>,
    xname_vec_opt: Option<Vec<&str>>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    match self {
      CSM(b) => {
        b.get_and_filter_sessions(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec_opt,
          xname_vec_opt,
          min_age_opt,
          max_age_opt,
          status_opt,
          cfs_session_name_opt,
          limit_number_opt,
          is_succeded_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_and_filter_sessions(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec_opt,
          xname_vec_opt,
          min_age_opt,
          max_age_opt,
          status_opt,
          cfs_session_name_opt,
          limit_number_opt,
          is_succeded_opt,
        )
        .await
      }
    }
  }

  async fn get_sessions_by_xname(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    xname_vec: &[&str],
    limit_opt: Option<u8>,
    after_id_opt: Option<String>,
    min_age_opt: Option<String>,
    max_age_opt: Option<String>,
    status_opt: Option<String>,
    name_contains_opt: Option<String>,
    is_succeded_opt: Option<bool>,
    tags_opt: Option<String>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    match self {
      CSM(b) => {
        b.get_sessions_by_xname(
          auth_token,
          base_url,
          root_cert,
          xname_vec,
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
      OCHAMI(b) => {
        b.get_sessions_by_xname(
          auth_token,
          base_url,
          root_cert,
          xname_vec,
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
    }
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
    match self {
      CSM(b) => {
        b.delete_and_cancel_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          group_available_vec,
          cfs_session,
          cfs_component_vec,
          bss_bootparameter_vec,
          dry_run,
        )
        .await
      }
      OCHAMI(b) => {
        b.delete_and_cancel_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          group_available_vec,
          cfs_session,
          cfs_component_vec,
          bss_bootparameter_vec,
          dry_run,
        )
        .await
      }
    }
  }

  async fn create_configuration_from_repos(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_root_cert: &[u8],
    repo_name_vec: Vec<String>,
    local_git_commit_vec: Vec<String>,
    playbook_file_name_opt: Option<&String>,
  ) -> Result<CfsConfigurationRequest, Error> {
    match self {
      CSM(b) => {
        b.create_configuration_from_repos(
          gitea_token,
          gitea_base_url,
          shasta_root_cert,
          repo_name_vec,
          local_git_commit_vec,
          playbook_file_name_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.create_configuration_from_repos(
          gitea_token,
          gitea_base_url,
          shasta_root_cert,
          repo_name_vec,
          local_git_commit_vec,
          playbook_file_name_opt,
        )
        .await
      }
    }
  }

  async fn get_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    cfs_configuration_name_opt: Option<&String>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    match self {
      CSM(b) => {
        b.get_configuration(
          auth_token,
          base_url,
          root_cert,
          cfs_configuration_name_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_configuration(
          auth_token,
          base_url,
          root_cert,
          cfs_configuration_name_opt,
        )
        .await
      }
    }
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
    match self {
      CSM(b) => {
        b.get_and_filter_configuration(
          auth_token,
          base_url,
          root_cert,
          configuration_name,
          configuration_name_pattern,
          hsm_group_name_vec,
          since_opt,
          until_opt,
          limit_number_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_and_filter_configuration(
          auth_token,
          base_url,
          root_cert,
          configuration_name,
          configuration_name_pattern,
          hsm_group_name_vec,
          since_opt,
          until_opt,
          limit_number_opt,
        )
        .await
      }
    }
  }

  async fn get_configuration_layer_details(
    &self,
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
    site_name: &str,
  ) -> Result<LayerDetails, Error> {
    match self {
      CSM(b) => {
        b.get_configuration_layer_details(
          shasta_root_cert,
          gitea_base_url,
          gitea_token,
          layer,
          site_name,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_configuration_layer_details(
          shasta_root_cert,
          gitea_base_url,
          gitea_token,
          layer,
          site_name,
        )
        .await
      }
    }
  }

  async fn update_runtime_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    xnames: Vec<String>,
    desired_configuration: &str,
    enabled: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.update_runtime_configuration(
          auth_token,
          base_url,
          root_cert,
          xnames,
          desired_configuration,
          enabled,
        )
        .await
      }
      OCHAMI(b) => {
        b.update_runtime_configuration(
          auth_token,
          base_url,
          root_cert,
          xnames,
          desired_configuration,
          enabled,
        )
        .await
      }
    }
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
    match self {
      CSM(b) => {
        b.put_configuration(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          configuration,
          configuration_name,
          overwrite,
        )
        .await
      }
      OCHAMI(b) => {
        b.put_configuration(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          configuration,
          configuration_name,
          overwrite,
        )
        .await
      }
    }
  }

  // Get all CFS sessions, IMS images and BOS sessiontemplates related to a CFS configuration
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
    match self {
      CSM(b) => {
        b.get_derivatives(auth_token, base_url, root_cert, configuration_name)
          .await
      }
      OCHAMI(b) => {
        b.get_derivatives(auth_token, base_url, root_cert, configuration_name)
          .await
      }
    }
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
    match self {
      CSM(b) => {
        b.get_cfs_components(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          configuration_name,
          components_ids,
          status,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_cfs_components(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          configuration_name,
          components_ids,
          status,
        )
        .await
      }
    }
  }
}

impl SatTrait for StaticBackendDispatcher {
  async fn apply_sat_file(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    k8s_api_url: &str,
    shasta_k8s_secrets: serde_json::Value,
    sat_template_file_yaml: serde_yaml::Value,
    hsm_group_available_vec: &Vec<String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    gitea_base_url: &str,
    gitea_token: &str,
    do_not_reboot: bool,
    watch_logs: bool,
    debug_on_failure: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.apply_sat_file(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          vault_base_url,
          vault_secret_path,
          k8s_api_url,
          shasta_k8s_secrets,
          sat_template_file_yaml,
          hsm_group_available_vec,
          ansible_verbosity_opt,
          ansible_passthrough_opt,
          gitea_base_url,
          gitea_token,
          do_not_reboot,
          watch_logs,
          debug_on_failure,
          overwrite,
          dry_run,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_sat_file(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          vault_base_url,
          vault_secret_path,
          k8s_api_url,
          shasta_k8s_secrets,
          sat_template_file_yaml,
          hsm_group_available_vec,
          ansible_verbosity_opt,
          ansible_passthrough_opt,
          gitea_base_url,
          gitea_token,
          do_not_reboot,
          watch_logs,
          debug_on_failure,
          overwrite,
          dry_run,
        )
        .await
      }
    }
  }
}

impl ApplyHwClusterPin for StaticBackendDispatcher {
  async fn apply_hw_cluster_pin(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
    nodryrun: bool,
    create_target_hsm_group: bool,
    delete_empty_parent_hsm_group: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.apply_hw_cluster_pin(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          parent_hsm_group_name,
          pattern,
          nodryrun,
          create_target_hsm_group,
          delete_empty_parent_hsm_group,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_hw_cluster_pin(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          parent_hsm_group_name,
          pattern,
          nodryrun,
          create_target_hsm_group,
          delete_empty_parent_hsm_group,
        )
        .await
      }
    }
  }
}

impl ImsTrait for StaticBackendDispatcher {
  async fn get_images(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    match self {
      CSM(b) => {
        b.get_images(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_images(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id_opt,
        )
        .await
      }
    }
  }

  async fn get_all_images(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
  ) -> Result<Vec<Image>, Error> {
    match self {
      CSM(b) => {
        b.get_all_images(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
      OCHAMI(b) => {
        b.get_all_images(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
    }
  }

  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    match self {
      CSM(b) => b.filter_images(image_vec),
      OCHAMI(b) => b.filter_images(image_vec),
    }
  }

  async fn delete_image(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_id: &str,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.delete_image(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id,
        )
        .await
      }
      OCHAMI(b) => {
        b.delete_image(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id,
        )
        .await
      }
    }
  }
}

impl ApplySessionTrait for StaticBackendDispatcher {
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_conf_sess_name: Option<&String>,
    playbook_yaml_file_name_opt: Option<&String>,
    hsm_group: Option<&String>,
    repos_name_vec: Vec<String>,
    repos_last_commit_id_vec: Vec<String>,
    ansible_limit: Option<String>,
    ansible_verbosity: Option<String>,
    ansible_passthrough: Option<String>,
  ) -> Result<(String, String), Error> {
    match self {
      CSM(b) => {
        b.apply_session(
          gitea_token,
          gitea_base_url,
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          cfs_conf_sess_name,
          playbook_yaml_file_name_opt,
          hsm_group,
          repos_name_vec,
          repos_last_commit_id_vec,
          ansible_limit,
          ansible_verbosity,
          ansible_passthrough,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_session(
          gitea_token,
          gitea_base_url,
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          cfs_conf_sess_name,
          playbook_yaml_file_name_opt,
          hsm_group,
          repos_name_vec,
          repos_last_commit_id_vec,
          ansible_limit,
          ansible_verbosity,
          ansible_passthrough,
        )
        .await
      }
    }
  }
}

impl MigrateRestoreTrait for StaticBackendDispatcher {
  async fn migrate_restore(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: Option<&String>,
    cfs_file: Option<&String>,
    hsm_file: Option<&String>,
    ims_file: Option<&String>,
    image_dir: Option<&String>,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.migrate_restore(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_file,
          cfs_file,
          hsm_file,
          ims_file,
          image_dir,
        )
        .await
      }
      OCHAMI(b) => {
        b.migrate_restore(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_file,
          cfs_file,
          hsm_file,
          ims_file,
          image_dir,
        )
        .await
      }
    }
  }
}

impl MigrateBackupTrait for StaticBackendDispatcher {
  async fn migrate_backup(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos: Option<&String>,
    destination: Option<&String>,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.migrate_backup(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos,
          destination,
        )
        .await
      }
      OCHAMI(b) => {
        b.migrate_backup(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos,
          destination,
        )
        .await
      }
    }
  }
}

impl GetImagesAndDetailsTrait for StaticBackendDispatcher {
  async fn get_images_and_details(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &[String],
    id_opt: Option<&String>,
    limit_number: Option<&u8>,
  ) -> Result<Vec<(Image, String, String, bool)>, Error> {
    match self {
      CSM(b) => {
        b.get_images_and_details(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          id_opt,
          limit_number,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_images_and_details(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          id_opt,
          limit_number,
        )
        .await
      }
    }
  }
}

impl ClusterSessionTrait for StaticBackendDispatcher {
  async fn post_template_session(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session: types::bos::session::BosSession,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => {
        b.post_template_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session,
        )
        .await
      }
      OCHAMI(b) => {
        b.post_template_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session,
        )
        .await
      }
    }
  }
}

impl ClusterTemplateTrait for StaticBackendDispatcher {
  async fn get_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    match self {
      CSM(b) => {
        b.get_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session_template_id_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session_template_id_opt,
        )
        .await
      }
    }
  }

  async fn get_and_filter_templates(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &Vec<String>,
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    match self {
      CSM(b) => {
        b.get_and_filter_templates(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          hsm_member_vec,
          bos_sessiontemplate_name_opt,
          limit_number_opt,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_and_filter_templates(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          hsm_member_vec,
          bos_sessiontemplate_name_opt,
          limit_number_opt,
        )
        .await
      }
    }
  }

  async fn get_all_templates(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    match self {
      CSM(b) => {
        b.get_all_templates(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
      OCHAMI(b) => {
        b.get_all_templates(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
    }
  }

  async fn put_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    match self {
      CSM(b) => {
        b.put_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_template,
          bos_template_name,
        )
        .await
      }
      OCHAMI(b) => {
        b.put_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_template,
          bos_template_name,
        )
        .await
      }
    }
  }

  async fn delete_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_template_id: &str,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.delete_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_template_id,
        )
        .await
      }
      OCHAMI(b) => {
        b.delete_template(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_template_id,
        )
        .await
      }
    }
  }
}

impl CommandsTrait for StaticBackendDispatcher {
  async fn i_delete_data_related_to_cfs_configuration(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_name_available_vec: Vec<String>,
    configuration_name_opt: Option<&String>,
    configuration_name_pattern: Option<&String>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    assume_yes: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.i_delete_data_related_to_cfs_configuration(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_name_available_vec,
          configuration_name_opt,
          configuration_name_pattern,
          since_opt,
          until_opt,
          assume_yes,
        )
        .await
      }
      OCHAMI(b) => {
        b.i_delete_data_related_to_cfs_configuration(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_name_available_vec,
          configuration_name_opt,
          configuration_name_pattern,
          since_opt,
          until_opt,
          assume_yes,
        )
        .await
      }
    }
  }
}

impl ConsoleTrait for StaticBackendDispatcher {
  type T = Box<dyn AsyncWrite + Unpin>;
  type U = Box<dyn AsyncRead + Unpin>;

  async fn attach_to_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    xname: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    match self {
      CSM(b) => {
        b.attach_to_console(shasta_token, site_name, xname, width, height, k8s)
          .await
      }
      OCHAMI(b) => {
        b.attach_to_console(shasta_token, site_name, xname, width, height, k8s)
          .await
      }
    }
  }
}
