use std::{
    error::Error,
    io::{stdout, Read, Write},
};

use futures_util::StreamExt;

use mesa::manta::console::get_container_attachment;
use termion::{color, raw::IntoRawMode};
use tokio::{io::AsyncWriteExt, runtime::Runtime};
use tokio_util::io::ReaderStream;

use crate::common::node_ops;

pub async fn exec(
    hsm_group: Option<&String>,
    // cli_console: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    xnames: Vec<&str>,
) {

    if hsm_group.is_some() {

        // Check user has provided valid XNAMES
        if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group).await {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }

    } else {
        // no hsm_group value provided
        // included = xnames.clone();
    }

    connect_to_console(
        // included.iter().next().unwrap(),
        &xnames.first().unwrap().to_string(),
        vault_base_url,
        vault_role_id,
        k8s_api_url,
    )
    .await
    .unwrap();
}

pub async fn connect_to_console(
    xname: &String,
    vault_base_url: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
) -> Result<(), Box<dyn Error>> {
    log::info!("xname: {}", xname);

    let mut attached =
        get_container_attachment(xname, vault_base_url, vault_role_id, k8s_api_url).await;

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
