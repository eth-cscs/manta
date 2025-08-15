use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    bss::BootParametersTrait, cfs::CfsTrait,
    hsm::group::GroupTrait,
  },
};
use std::time::Instant;

pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  group_available_vec: Vec<String>,
  session_name: &str,
  dry_run: bool,
  assume_yes: bool,
) -> Result<(), Error> {
  // Get collectives (CFS configuration, CFS session, BOS session template, IMS image and CFS component)
  let start = Instant::now();
  log::info!("Fetching data from the backend...");
  let (
    group_available_vec,
    cfs_session_vec,
    cfs_component_vec,
    bss_bootparameters_vec,
  ) = tokio::try_join!(
    backend.get_group_available(shasta_token),
    backend.get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      Some(group_available_vec),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    ),
    backend.get_cfs_components(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
      None,
      None,
    ),
    backend.get_all_bootparameters(shasta_token,),
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
    .find(|cfs_session| cfs_session.name.eq(&Some(session_name.to_string())))
    .ok_or_else(|| {
      Error::Message(format!("CFS session '{}' not found. Exit", session_name))
    })?;

  if !assume_yes {
    // Ask user for confirmation
    let user_msg = format!(
      "Session '{}' will get canceled:\nDo you want to continue?",
      session_name,
    );
    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt(user_msg)
      .interact()
      .unwrap()
    {
      log::info!("Continue",);
    } else {
      println!("Cancelled by user. Aborting.");
      return Ok(());
    }
  }

  let image_created_by_cfs_session_vec = cfs_session.get_result_id_vec();
  if !image_created_by_cfs_session_vec.is_empty() {
    if !assume_yes {
      // Ask user for confirmation
      let user_msg = format!(
                    "Images listed below which will get deleted:\n{}\nDo you want to continue?",
                    image_created_by_cfs_session_vec.join("\n"),
                );
      if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(user_msg)
        .interact()
        .unwrap()
      {
        log::info!("Continue",);
      } else {
        println!("Cancelled by user. Aborting.");
        return Ok(());
      }
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
    .await
}
