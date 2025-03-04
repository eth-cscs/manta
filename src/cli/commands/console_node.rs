use std::collections::HashMap;

use futures::StreamExt;

use mesa::node::{self, console};
use termion::color;
use tokio::{io::AsyncWriteExt, select};

use crate::{cli::commands::config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all, common::{self, terminal_ops}};

use super::power_on_nodes::is_user_input_nids;

pub async fn exec(
    hsm_group: Option<&String>,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    host: &str,
) {
    let hsm_name_available_vec =
        get_hsm_name_without_system_wide_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    // Get HSM group user has access to
    let hsm_group_available_map = mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_without_system_wide_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_name_available_vec
            .iter()
            .map(|hsm_name| hsm_name.as_str())
            .collect(),
    )
    .await
    .expect("ERROR - could not get HSM group summary");

    // Filter xnames to the ones members to HSM groups the user has access to
    //
    /* let _ = mesa::hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
        .await; */

    // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
    let mut xname_vec = if is_user_input_nids(host) {
        log::debug!("User input seems to be NID");
        common::node_ops::nid_to_xname(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            host,
            false,
        )
        .await
        .expect("Could not convert NID to XNAME")
    } else {
        log::debug!("User input seems to be XNAME");
        let hsm_group_summary: HashMap<String, Vec<String>> = 
            // Get HashMap with HSM groups and members curated for this request.
            // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
            // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
            // hostlist have been removed
            common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                &host,
            hsm_group_available_map,
            false,
            )
            .await;

        hsm_group_summary.values().flatten().cloned().collect()
    };

    xname_vec.dedup();

    if xname_vec.is_empty() {
        eprintln!("ERROR - node '{}' not found", host);
    }

    log::debug!("input {} translates to xname {:?}", host, xname_vec);

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
        // included.iter().next().unwrap(),
        &xname.to_string(),
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
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
    xname: &String,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
) -> Result<(), anyhow::Error> {
    log::info!("xname: {}", xname);

    let mut attached = console::get_container_attachment_to_conman(
        xname,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
    )
    .await;

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

    let mut output = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let mut input = attached.stdin().unwrap();

    let term_tx = attached.terminal_size().unwrap();

    let mut handle_terminal_size_handle = tokio::spawn(terminal_ops::handle_terminal_size(term_tx));

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

    /* let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_stream = ReaderStream::new(attached.stdout().unwrap());

    let mut stdin = std::io::stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        let mut next_stdout;

        loop {
            next_stdout = stdout_stream.next().await;
            match next_stdout {
                Some(next_from_remote_stdout) => {
                    // Print stream to stdout while steam lives
                    match next_from_remote_stdout {
                        Ok(remote_stdout) => {
                            print!("{}", String::from_utf8_lossy(&remote_stdout));
                            stdout.flush().unwrap();
                        }
                        Err(e) => {
                            log::warn!("There was an error reading stream input");
                            eprintln!("{:?}", e);
                            stdout.suspend_raw_mode().unwrap();
                            // std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Stream has finished. Reseting terminal and Exiting application.
                    stdout.suspend_raw_mode().unwrap();
                    std::process::exit(0);
                }
            }
        }
    });

    loop {
        let mut buffer = [0; 1];

        let n = stdin.read(&mut buffer[..])?;

        stdin_writer
            .write_all(
                String::from_utf8(buffer[..n].to_vec())
                    .unwrap()
                    .to_string()
                    .as_bytes(),
            )
            .await?;
    } */
}
