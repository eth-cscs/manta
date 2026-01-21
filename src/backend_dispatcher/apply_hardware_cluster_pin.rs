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
