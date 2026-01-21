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

impl ApplySessionTrait for StaticBackendDispatcher {
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    hsm_group: Option<&str>,
    repos_name_vec: &[&str],
    repos_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
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
