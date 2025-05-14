use futures::StreamExt;
use manta_backend_dispatcher::{
  interfaces::cfs::CfsTrait,
  types::{K8sAuth, K8sDetails},
};

use csm_rs::node::console;
use termion::color;
use tokio::{io::AsyncWriteExt, select};

use crate::{
  common::{
    terminal_ops, vault::http_client::fetch_shasta_k8s_secrets_from_vault,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  hsm_group_name_vec: &Vec<String>,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  k8s_api_url: &str,
  cfs_session_name: &str,
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
      Some(&cfs_session_name.to_string()),
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

  connect_to_console(
    site_name,
    shasta_token,
    &cfs_session_name.to_string(),
    /* vault_base_url,
    vault_secret_path,
    vault_role_id, */
    k8s_api_url,
    k8s,
  )
  .await
  .unwrap();
}

pub async fn connect_to_console(
  site_name: &str,
  shasta_token: &str,
  cfs_session_name: &String,
  /* vault_base_url: &str,
  vault_secret_path: &str,
  vault_role_id: &str, */
  k8s_api_url: &str,
  k8s: &K8sDetails,
) -> Result<(), anyhow::Error> {
  log::info!("CFS session name: {}", cfs_session_name);

  let shasta_k8s_secrets = match &k8s.authentication {
    K8sAuth::Native {
      certificate_authority_data,
      client_certificate_data,
      client_key_data,
    } => {
      serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
    }
    K8sAuth::Vault {
      base_url,
      // secret_path: _secret_path,
    } => {
      fetch_shasta_k8s_secrets_from_vault(&base_url, site_name, shasta_token)
        .await
        .unwrap()
    }
  };

  let mut attached =
    console::get_container_attachment_to_cfs_session_image_target(
      cfs_session_name,
      /* vault_base_url,
      vault_secret_path,
      vault_role_id, */
      k8s_api_url,
      shasta_k8s_secrets,
    )
    .await?;

  println!(
    "Connected to {}{}{}!",
    color::Fg(color::Blue),
    cfs_session_name,
    color::Fg(color::Reset)
  );
  println!(
    "Use {}&.{} key combination to exit the console.",
    color::Fg(color::Green),
    color::Fg(color::Reset)
  );

  let mut stdin = tokio_util::io::ReaderStream::new(tokio::io::stdin());
  let mut stdout = tokio::io::stdout();

  let mut output =
    tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
  let mut input = attached.stdin().unwrap();

  let term_tx = attached.terminal_size().unwrap();

  let mut handle_terminal_size_handle =
    tokio::spawn(terminal_ops::handle_terminal_size(term_tx));

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
        result = &mut handle_terminal_size_handle => {
            match result {
                Ok(_) => log::info!("End of terminal size stream"),
                Err(e) => {
                    crossterm::terminal::disable_raw_mode()?;
                    log::error!("Error getting terminal size: {e:?}")
                }
            }
        },
    };
  }

  crossterm::terminal::disable_raw_mode()?;

  Ok(())
}
