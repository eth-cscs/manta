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

impl ConsoleTrait for StaticBackendDispatcher {
  type T = Box<dyn AsyncWrite + Unpin>;
  type U = Box<dyn AsyncRead + Unpin>;

  async fn attach_to_node_console(
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
        b.attach_to_node_console(
          shasta_token,
          site_name,
          xname,
          width,
          height,
          k8s,
        )
        .await
      }
      OCHAMI(b) => {
        b.attach_to_node_console(
          shasta_token,
          site_name,
          xname,
          width,
          height,
          k8s,
        )
        .await
      }
    }
  }

  async fn attach_to_session_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    session_name: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    match self {
      CSM(b) => {
        b.attach_to_session_console(
          shasta_token,
          site_name,
          session_name,
          width,
          height,
          k8s,
        )
        .await
      }
      OCHAMI(b) => {
        b.attach_to_session_console(
          shasta_token,
          site_name,
          session_name,
          width,
          height,
          k8s,
        )
        .await
      }
    }
  }
}
