use std::{
    error::Error,
    io::{stdout, Read, Write},
};

use futures::StreamExt;

use mesa::manta::console;
use termion::{color, raw::IntoRawMode};
use tokio::{io::AsyncWriteExt, runtime::Runtime};
use tokio_util::io::ReaderStream;

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
    ///////////////////////////////////////////
    // TODO: VALIDATE CFS SESSION NAME BELONGS TO THE HSM GROUP RESTRICTED TO THE USER
    ///////////////////////////////////////////
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
) -> Result<(), Box<dyn Error>> {
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

    let mut stdin_writer = attached.stdin().unwrap();
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
    }
}
