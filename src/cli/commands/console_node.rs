use futures::StreamExt;
use manta_backend_dispatcher::{
  interfaces::{console::ConsoleTrait, hsm::component::ComponentTrait},
  types::K8sDetails,
};

use tokio::{io::AsyncWriteExt, select};

use crate::{
  common::{self},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_token: &str,
  xname: &str,
  k8s: &K8sDetails,
) {
  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    xname,
    false,
    node_metadata_available_vec,
  )
  .await
  .unwrap_or_else(|e| {
    eprintln!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    );
    std::process::exit(1);
  });

  if xname_vec.len() != 1 {
    eprintln!("ERROR - The node to operate is not valid. Nothing to do. Exit");
    std::process::exit(0);
  }

  let xname = xname_vec.first().unwrap();

  let console_rslt = connect_to_console(
    backend,
    shasta_token,
    site_name,
    // included.iter().next().unwrap(),
    &xname.to_string(),
    // k8s_api_url,
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
  xname: &String,
  // k8s_api_url: &str,
  k8s: &K8sDetails,
) -> Result<(), anyhow::Error> {
  log::info!("xname: {}", xname);

  let (width, height) = crossterm::terminal::size()?;

  let (a_input, a_output) = backend
    .attach_to_console(shasta_token, site_name, xname, width, height, &k8s)
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

        /* result = &mut handle_terminal_size_handle => {
            match result {
                Ok(_) => log::info!("End of terminal size stream"),
                Err(e) => {
                    crossterm::terminal::disable_raw_mode()?;
                    log::error!("Error getting terminal size: {e:?}")
                }
            }
        }, */
    };
  }

  crossterm::terminal::disable_raw_mode()?;

  Ok(())
}
