use backend_dispatcher::{
    interfaces::{console::ConsoleTrait, hsm::component::ComponentTrait},
    types::{K8sAuth, K8sDetails},
};
use futures::StreamExt;

use mesa::node::{self, console};
use termion::color;
use tokio::{io::AsyncWriteExt, select};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, terminal_ops, vault::http_client::fetch_shasta_k8s_secrets_from_vault},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    site_name: &str,
    hsm_group: Option<&String>,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    // k8s_api_url: &str,
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

    let mut xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
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

    xname_vec.dedup();

    let xname = xname_vec.first().unwrap();

    if hsm_group.is_some() {
        // Check user has provided valid XNAMES
        if !node::utils::validate_xnames_format_and_membership_agaisnt_single_hsm(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &[xname],
            hsm_group,
        )
        .await
        {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }
    } else {
        // no hsm_group value provided
        // included = xnames.clone();
        node::utils::validate_xname_format(xname);
    }

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

    /* let shasta_k8s_secrets =
    fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await?; */

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
        } => fetch_shasta_k8s_secrets_from_vault(&base_url, &site_name, shasta_token)
            .await
            .unwrap(),
    };

    /* let mut attached =
        console::get_container_attachment_to_conman(xname, &k8s.api_url, shasta_k8s_secrets)
            .await?;

    println!(
        "Connected to {}{}{}!",
        color::Fg(color::Blue),
        xname,
        color::Fg(color::Reset)
    );
    println!(
        "Use {}&.{} key combination to exit the console.",
        color::Fg(color::Green),
        color::Fg(color::Reset)
    );

    println!("Connected to {}!", xname,);
    println!("Use &. key combination to exit the console.",);

    let mut stdin = tokio_util::io::ReaderStream::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();

    let mut input = attached.stdin().unwrap();
    let mut output = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());

    let term_tx = attached.terminal_size().unwrap();

    let mut handle_terminal_size_handle = tokio::spawn(terminal_ops::handle_terminal_size(term_tx)); */

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
