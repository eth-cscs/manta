use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::{cfs::CfsTrait, console::ConsoleTrait},
  types::K8sDetails,
};

use crate::{
  common::{
    authentication::get_api_token, authorization::get_groups_names_available,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use futures::StreamExt;
use tokio::{io::AsyncWriteExt, select};

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
    bail!("No CFS session found. Exit");
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
       'image'. Exit",
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
       'running'. Exit",
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
       to any availble HSM groups {:?}. Exit",
      cfs_session_details.name,
      hsm_group_name_vec
    );
  }

  let console_rslt = connect_to_console(
    backend,
    &shasta_token,
    site_name,
    &session_name.to_string(),
    k8s,
  )
  .await;

  match console_rslt {
    Ok(_) => {
      let _ = crossterm::terminal::disable_raw_mode();
      log::info!("Console closed");
    }
    Err(error) => {
      let _ = crossterm::terminal::disable_raw_mode();
      log::error!("{:?}", error);
    }
  }

  Ok(())
}

async fn connect_to_console(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  site_name: &str,
  session_name: &String,
  k8s: &K8sDetails,
) -> Result<(), anyhow::Error> {
  log::info!("session: {}", session_name);

  let (width, height) = crossterm::terminal::size()?;

  let (a_input, a_output) = backend
    .attach_to_session_console(
      shasta_token,
      site_name,
      session_name,
      width,
      height,
      k8s,
    )
    .await?;

  let mut stdin = tokio_util::io::ReaderStream::new(tokio::io::stdin());
  let mut stdout = tokio::io::stdout();

  let mut output = tokio_util::io::ReaderStream::new(a_output);
  let mut input = a_input;

  crossterm::terminal::enable_raw_mode()?;

  loop {
    select! {
        message = stdin.next() => {
            match message {
                Some(Ok(message)) => {
                    input.write_all(&message).await?;
                },
                Some(Err(message)) => {
                   crossterm::terminal::disable_raw_mode()?;
                   log::error!("ERROR: Console stdin {:?}", &message);
                   break
                },
                None => {
                    crossterm::terminal::disable_raw_mode()?;
                    log::info!("NONE (No input): Console stdin");
                    break
                },
            }
        },

        message = output.next() => {
            match message {
                Some(Ok(message)) => {
                    stdout.write_all(&message).await?;
                    stdout.flush().await?;
                },
                Some(Err(message)) => {
                   crossterm::terminal::disable_raw_mode()?;
                   log::error!("ERROR: Console stdout: {:?}", &message);
                   break
                },
                None => {
                    crossterm::terminal::disable_raw_mode()?;
                    log::info!("Exit console");
                    break
                },
            }
        },
    };
  }

  crossterm::terminal::disable_raw_mode()?;

  Ok(())
}
