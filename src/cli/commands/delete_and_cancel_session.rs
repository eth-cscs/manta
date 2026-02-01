use crate::common::{
  self, authentication::get_api_token, authorization::get_groups_names_available,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

use manta_backend_dispatcher::interfaces::{
  bss::BootParametersTrait, cfs::CfsTrait, hsm::group::GroupTrait,
};
use std::time::Instant;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  settings_hsm_group_name_opt: Option<&String>,
  session_name: &str,
  dry_run: bool,
  assume_yes: bool,
) -> Result<(), anyhow::Error> {
  let shasta_token = get_api_token(backend, site_name).await?;
  let group_available_vec = get_groups_names_available(
    backend,
    &shasta_token,
    None,
    settings_hsm_group_name_opt,
  )
  .await?;

  // Get collectives (CFS configuration, CFS session, BOS session template, IMS image and CFS component)
  let start = Instant::now();
  log::info!("Fetching data from the backend...");
  let (
    group_available_vec,
    cfs_session_vec,
    cfs_component_vec,
    bss_bootparameters_vec,
  ) = tokio::try_join!(
    backend.get_group_available(&shasta_token),
    backend.get_and_filter_sessions(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      group_available_vec,
      Vec::new(),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    ),
    backend.get_cfs_components(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
      None,
      None,
    ),
    backend.get_all_bootparameters(&shasta_token,),
  )?;

  let duration = start.elapsed();
  log::info!(
    "Time elapsed to fetch information from backend: {:?}",
    duration
  );

  // Validate:
  // - Check CFS session to delete exists
  // - CFS configuration related to CFS session is not being used to create an image
  // - CFS configuration related to CFS session is not a desired configuration
  //
  // Get CFS session to delete
  // let mut cfs_session_vec = cfs_session_vec_opt.unwrap_or_default();

  // Check CFS session to delete exists (filter sessions by name)
  let cfs_session = cfs_session_vec
    .iter()
    .find(|cfs_session| cfs_session.name.eq(&session_name.to_string()))
    .ok_or_else(|| {
      anyhow::Error::msg(format!("CFS session '{}' not found. Exit", session_name))
    })?;

  if !common::user_interaction::confirm(
    &format!(
      "Session '{}' will get canceled:\nDo you want to continue?",
      session_name,
    ),
    assume_yes,
  ) {
    println!("Cancelled by user. Aborting.");
    return Ok(());
  }

  let image_created_by_cfs_session_vec = cfs_session.get_result_id_vec();
  if !image_created_by_cfs_session_vec.is_empty() {
    if !common::user_interaction::confirm(
      &format!(
        "Images listed below which will get deleted:\n{}\nDo you want to continue?",
        image_created_by_cfs_session_vec.join("\n"),
      ),
      assume_yes,
    ) {
      println!("Cancelled by user. Aborting.");
      return Ok(());
    }
  }

  backend
    .delete_and_cancel_session(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &group_available_vec,
      cfs_session,
      &cfs_component_vec,
      &bss_bootparameters_vec,
      dry_run,
    )
    .await?;

  Ok(())
}
