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
    hsm_group_name_vec: &[String],
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&str>,
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
