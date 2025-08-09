use crate::{
  common::authorization::get_groups_available,
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use chrono::NaiveDateTime;
use manta_backend_dispatcher::{
  error::Error, interfaces::commands::CommandsTrait,
};

pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  target_hsm_group_vec: Vec<String>,
  configuration_name_opt: Option<&String>,
  configuration_name_pattern: Option<&String>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  assume_yes: bool,
) -> Result<(), Error> {
  backend
    .i_delete_data_related_to_cfs_configuration(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_vec,
      configuration_name_opt,
      configuration_name_pattern,
      since_opt,
      until_opt,
      assume_yes,
    )
    .await
}
