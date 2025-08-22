use manta_backend_dispatcher::{
  interfaces::{cfs::CfsTrait, console::ConsoleTrait},
  types::K8sDetails,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

use futures::StreamExt;
use tokio::{io::AsyncWriteExt, select};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  hsm_group_name_vec: &Vec<String>,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  session_name: &str,
  k8s: &K8sDetails,
) {
  let cfs_session_vec = backend
    .get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
      None,
      None,
      None,
      None,
      Some(&session_name.to_string()),
      None,
      None,
    )
    .await
    .unwrap_or_else(|e| {
      log::error!("Failed to get CFS sessions. Reason:\n{e}");
      std::process::exit(1);
    });

  if cfs_session_vec.is_empty() {
    eprintln!("No CFS session found. Exit",);
    std::process::exit(1);
  }

  let cfs_session_details = cfs_session_vec.first().unwrap();

  if cfs_session_details
    .target
    .as_ref()
    .unwrap()
    .definition
    .as_ref()
    .unwrap()
    .ne("image")
  {
    eprintln!(
      "CFS session found {} is type dynamic. Exit",
      cfs_session_details.name.as_ref().unwrap()
    );
    std::process::exit(1);
  }

  if cfs_session_details
    .status
    .as_ref()
    .unwrap()
    .session
    .as_ref()
    .unwrap()
    .status
    .ne(&Some("running".to_string()))
  {
    eprintln!(
      "CFS session found {} state is not 'running'. Exit",
      cfs_session_details.name.as_ref().unwrap()
    );
    std::process::exit(1);
  }

  if !cfs_session_details
    .target
    .as_ref()
    .unwrap()
    .groups
    .as_ref()
    .unwrap()
    .iter()
    .any(|group| hsm_group_name_vec.contains(&group.name.to_string()))
  {
    eprintln!(
      "CFS session found {} is not related to any availble HSM groups {:?}",
      cfs_session_details.name.as_ref().unwrap(),
      hsm_group_name_vec
    );
    std::process::exit(1);
  }

  let console_rslt = connect_to_console(
    backend,
    shasta_token,
    site_name,
    &session_name.to_string(),
    k8s,
  )
  .await;

  match console_rslt {
    Ok(_) => {
      crossterm::terminal::disable_raw_mode().unwrap();
      log::info!("Console closed");
    }
    Err(error) => {
      crossterm::terminal::disable_raw_mode().unwrap();
      log::error!("{:?}", error);
    }
  }
}

pub async fn connect_to_console(
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
      &session_name,
      width,
      height,
      &k8s,
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
