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

impl ImsTrait for StaticBackendDispatcher {
  async fn get_images(
    &self,
    shasta_token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    match self {
      CSM(b) => b.get_images(shasta_token, image_id_opt).await,
      OCHAMI(b) => b.get_images(shasta_token, image_id_opt).await,
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

  async fn update_image(
    &self,
    shasta_token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.update_image(shasta_token, image_id, image).await,
      OCHAMI(b) => b.update_image(shasta_token, image_id, image).await,
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
