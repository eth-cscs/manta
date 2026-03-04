use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::{cfs::CfsTrait, console::ConsoleTrait},
  types::K8sDetails,
};

use crate::{
  cli::commands::console_common,
  common::{
    authentication::get_api_token, authorization::get_groups_names_available,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  settings_hsm_group_name_opt: Option<&String>,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  session_name: &str,
  k8s: &K8sDetails,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  let hsm_group_name_vec = get_groups_names_available(
    backend,
    &shasta_token,
    None,
    settings_hsm_group_name_opt,
  )
  .await?;

  let cfs_session_vec = backend
    .get_and_filter_sessions(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      Vec::new(),
      Vec::new(),
      None,
      None,
      None,
      None,
      Some(&session_name.to_string()),
      None,
      None,
    )
    .await
    .map_err(|e| {
      Error::msg(format!("Failed to get CFS sessions. Reason:\n{e}"))
    })?;

  if cfs_session_vec.is_empty() {
    bail!("No CFS session found");
  }

  let cfs_session_details =
    cfs_session_vec.first().context("No CFS session found")?;

  if cfs_session_details
    .target
    .as_ref()
    .context("CFS session target is missing")?
    .definition
    .as_ref()
    .context("CFS session target definition is missing")?
    .ne("image")
  {
    bail!(
      "CFS session found {} is type not \
       'image'",
      cfs_session_details.name
    );
  }

  if cfs_session_details
    .status
    .as_ref()
    .context("CFS session status is missing")?
    .session
    .as_ref()
    .context("CFS session status session is missing")?
    .status
    .ne(&Some("running".to_string()))
  {
    bail!(
      "CFS session found {} state is not \
       'running'",
      cfs_session_details.name
    );
  }

  if !cfs_session_details
    .target
    .as_ref()
    .context("CFS session target is missing")?
    .groups
    .as_ref()
    .context("CFS session target groups is missing")?
    .iter()
    .any(|group| hsm_group_name_vec.contains(&group.name.to_string()))
  {
    bail!(
      "CFS session found {} is not related \
       to any available HSM groups {:?}",
      cfs_session_details.name,
      hsm_group_name_vec
    );
  }

  log::info!("session: {}", session_name);

  let (width, height) = crossterm::terminal::size()?;

  let (a_input, a_output) = backend
    .attach_to_session_console(
      &shasta_token,
      site_name,
      session_name,
      width,
      height,
      k8s,
    )
    .await?;

  let result = console_common::run_console_loop(a_input, a_output).await;

  console_common::handle_console_result(result);

  Ok(())
}
