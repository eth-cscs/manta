use futures::StreamExt;

use mesa::manta::console;
use termion::color;
use tokio::{io::AsyncWriteExt, select};

use crate::common::terminal_ops;

pub async fn exec(
    hsm_group: Option<&String>,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    cfs_session_name: &str,
) {
    let cfs_session_details_list_rslt = mesa::shasta::cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group,
        Some(&cfs_session_name.to_string()),
        None,
        Some(false),
    )
    .await;
    if let Ok(cfs_session_details_list) = cfs_session_details_list_rslt {
        if cfs_session_details_list.is_empty() {
            eprintln!("No CFS session found. Exit",);
            std::process::exit(1);
        }
        let cfs_session_details = cfs_session_details_list.first().unwrap();
        if cfs_session_details
            .pointer("/target/definition")
            .unwrap()
            .ne("image")
        {
            eprintln!(
                "CFS session found {} is type dynamic. Exit",
                cfs_session_details["name"].as_str().unwrap()
            );
            std::process::exit(1);
        }
        if cfs_session_details
            .pointer("/status/session/status")
            .unwrap()
            .as_str()
            .ne(&Some("running"))
        {
            eprintln!(
                "CFS session found {} state is not 'running'. Exit",
                cfs_session_details["name"].as_str().unwrap()
            );
            std::process::exit(1);
        }
        if cfs_session_details
            .pointer("/target/groups")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["name"].as_str().unwrap().eq(hsm_group.unwrap()))
        {
            eprintln!(
                "CFS session found {} is not targeting HSM group {}",
                cfs_session_details["name"].as_str().unwrap(),
                hsm_group.unwrap()
            );
            std::process::exit(1);
        }
    }

    connect_to_console(
        &cfs_session_name.to_string(),
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
    )
    .await
    .unwrap();
}

pub async fn connect_to_console(
    cfs_session_name: &String,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
) -> Result<(), anyhow::Error> {
    log::info!("CFS session name: {}", cfs_session_name);

    let mut attached = console::get_container_attachment_to_cfs_session_image_target(
        cfs_session_name,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
    )
    .await;

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
                       input.write_all(format!("#*#* stdin {:?}", &message).as_bytes()).await?;
                       break
                    },
                    None => {
                        input.write_all("stdin None".as_bytes()).await?;
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
                       input.write_all(format!("#*#* stdout {:?}", &message).as_bytes()).await?;
                       break
                    },
                    None => {
                        input.write_all("stdout None".as_bytes()).await?;
                        break
                    },
                }
            },
            result = &mut handle_terminal_size_handle => {
                match result {
                    Ok(_) => println!("End of terminal size stream"),
                    Err(e) => println!("Error getting terminal size: {e:?}")
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
