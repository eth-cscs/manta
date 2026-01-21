use std::{collections::HashMap, pin::Pin};

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_hw_cluster_pin::ApplyHwClusterPin,
    apply_sat_file::SatTrait,
    apply_session::ApplySessionTrait,
    authentication::AuthenticationTrait,
    bos::{ClusterSessionTrait, ClusterTemplateTrait},
    bss::BootParametersTrait,
    cfs::CfsTrait,
    console::ConsoleTrait,
    delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait,
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
    self, Component, ComponentArrayPostArray, Group, HWInventoryByLocationList,
    K8sDetails, NodeMetadataArray,
    bos::{session::BosSession, session_template::BosSessionTemplate},
    bss::BootParameters,
    cfs::{
      cfs_configuration_details::LayerDetails,
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::{CfsConfigurationResponse, Layer},
      component::Component as CfsComponent,
      session::{CfsSessionGetResponse, CfsSessionPostRequest},
    },
    hsm::inventory::{RedfishEndpoint, RedfishEndpointArray},
    ims::{Image, PatchImage},
  },
};

use StaticBackendDispatcher::*;
use chrono::NaiveDateTime;
use futures::AsyncBufRead;
use tokio::io::{AsyncRead, AsyncWrite};

use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;
use serde_json::Value;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
