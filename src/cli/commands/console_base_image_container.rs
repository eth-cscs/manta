use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;

use kube::{
    api::{
        Api, AttachParams, AttachedProcess, DeleteParams, PostParams, ResourceExt, WatchEvent,
        WatchParams,
    },
    Client,
};
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
    // Create pod based on Alps base image
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await.unwrap();

    let container_name = hsm_group.unwrap().to_owned() + "-alps-base-image-test";

    let p: Pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": { "name": container_name }, // Do we
                                                                                         // need to
                                                                                         // add
                                                                                         // namespace????
        "spec": {
            "containers": [{
                "name": container_name,
                "image": "artifactory.algol60.net/csm-docker/stable/cray-ims-sshd:1.6.1",
                "command": ["sh", "-ce", "/scripts/prep-env.sh /mnt/image", "https://rgw-vip.nmn/boot-images/4bf91021-8d99-4adf-945f-46de2ff50a3d/rootfs?AWSAccessKeyId=IZIUZCD5RF6ASL4PSZ4J&Signature=bgqyGD%2BFIXsnROhP1n1bHvnRH74%3D&Expires=1692908765"],
                "resources": {
                    "requests": {
                        "memory": "15Gi",
                        "cpu": "500m",
                    },
                    "limits": {
                        "memory": "45Gi",
                        "cpu": "8",
                    }
                },
                "env": [
                { "name": "CA_CERT", "value": "/etc/cray/ca/certificate_authority.crt" }, 
                { "name": "OAUTH_CONFIG_DIR", "value": "/etc/admin-client-auth" }
                ]
                
            }],
        }
    }))
    .unwrap();

    let pods: Api<Pod> = Api::default_namespaced(client);
    // Stop on error including a pod already exists or is still being deleted.
    pods.create(&PostParams::default(), &p).await.unwrap();

    // Wait until the pod is running, otherwise we get 500 error.
    let wp = WatchParams::default()
        .fields("metadata.name=example")
        .timeout(10);
    let mut stream = pods.watch(&wp, "0").await.unwrap().boxed();
    while let Some(status) = stream.try_next().await.unwrap() {
        match status {
            WatchEvent::Added(o) => {
                log::info!("Added {}", o.name_any());
            }
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                if s.phase.clone().unwrap_or_default() == "Running" {
                    log::info!("Ready to attach to {}", o.name_any());
                    break;
                }
            }
            _ => {}
        }
    }

    // TODO: when connecting to the container, show a banner saying the hardware the container ir
    // running on is not an ALPS compute node

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
                        input.write(&message).await?;
                    },
                    Some(Err(message)) => {
                       input.write(format!("#*#* stdin {:?}", &message).as_bytes()).await?;
                       break
                    },
                    None => {
                        input.write("stdin None".as_bytes()).await?;
                        break
                    },
                }
            },
            message = output.next() => {
                match message {
                    Some(Ok(message)) => {
                        stdout.write(&message).await?;
                        stdout.flush().await?;
                    },
                    Some(Err(message)) => {
                       input.write(format!("#*#* stdout {:?}", &message).as_bytes()).await?;
                       break
                    },
                    None => {
                        input.write("stdout None".as_bytes()).await?;
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
