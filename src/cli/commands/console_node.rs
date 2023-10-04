use futures::StreamExt;

use mesa::manta::console;
use termion::color;
use tokio::{io::AsyncWriteExt, select};

use mesa::common::node_ops;

use crate::common::terminal_ops;

pub async fn exec(
    hsm_group: Option<&String>,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    xname: &str,
) {
    if hsm_group.is_some() {
        // Check user has provided valid XNAMES
        if !node_ops::validate_xnames(shasta_token, shasta_base_url, &[xname], hsm_group).await {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }
    } else {
        // no hsm_group value provided
        // included = xnames.clone();
        node_ops::validate_xname_format(xname);
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
