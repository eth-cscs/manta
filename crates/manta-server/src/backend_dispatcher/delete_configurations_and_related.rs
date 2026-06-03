//! Dispatches `DeleteConfigurationsAndDataRelatedTrait` methods to csm-rs or ochami-rs.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait,
  types::cfs::{
    cfs_configuration_response::CfsConfigurationResponse,
    session::CfsSessionGetResponse,
  },
};

use StaticBackendDispatcher::*;
use chrono::NaiveDateTime;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl DeleteConfigurationsAndDataRelatedTrait for StaticBackendDispatcher {
  async fn get_data_to_delete(
    &self,
    shasta_token: &str,
    hsm_name_available_vec: &[String],
    configuration_name_pattern_opt: Option<&str>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
  ) -> Result<
    (
      Vec<CfsSessionGetResponse>,
      Vec<(String, String, String)>,
      Vec<String>,
      Vec<String>,
      Vec<(String, String, String)>,
      Vec<CfsConfigurationResponse>,
    ),
    Error,
  > {
    dispatch!(
      self,
      get_data_to_delete,
      shasta_token,
      hsm_name_available_vec,
      configuration_name_pattern_opt,
      since_opt,
      until_opt
    )
  }

  async fn delete(
    &self,
    shasta_token: &str,
    cfs_configuration_name_vec: &[String],
    image_id_vec: &[String],
    cfs_session_name_vec: &[String],
    bos_sessiontemplate_name_vec: &[String],
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete,
      shasta_token,
      cfs_configuration_name_vec,
      image_id_vec,
      cfs_session_name_vec,
      bos_sessiontemplate_name_vec
    )
  }
}
