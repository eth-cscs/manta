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

impl MigrateRestoreTrait for StaticBackendDispatcher {
  async fn migrate_restore(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
          overwrite_group,
          overwrite_configuration,
          overwrite_image,
          overwrite_template,
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
          overwrite_group,
          overwrite_configuration,
          overwrite_image,
          overwrite_template,
        )
        .await
      }
    }
  }
}
