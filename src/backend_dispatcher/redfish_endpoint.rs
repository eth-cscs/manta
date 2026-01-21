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
