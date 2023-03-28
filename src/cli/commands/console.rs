use std::{
    error::Error,
    io::{stdout, Read, Write},
};

use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::AttachParams, Api};
use serde_json::Value;
use termion::{color, raw::IntoRawMode};
use tokio::{io::AsyncWriteExt, runtime::Runtime};
use tokio_util::io::ReaderStream;

use crate::shasta::kubernetes::get_k8s_client_programmatically;

use crate::common::node_ops;

use clap::ArgMatches;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_console: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
) {
    /* let included: HashSet<String>;
    let excluded: HashSet<String>; */

    // User provided list of xnames to power reset
    let xnames: Vec<&str> = cli_console
        .get_one::<String>("XNAME")
        .unwrap()
        .split(',')
        .map(|xname| xname.trim())
        .collect();

    // let hsm_groups: Vec<cluster_ops::ClusterDetails>;

    if hsm_group.is_some() {
        /* // hsm_group value provided
        hsm_groups =
            cluster_ops::get_details(shasta_token, shasta_base_url, hsm_group.unwrap()).await; */

        /* // Take all nodes for all hsm_groups found and put them in a Set
        let hsm_groups_nodes: Vec<&str> = hsm_groups
            .iter()
            .flat_map(|hsm_group| {
                hsm_group
                    .members
                    .iter()
                    .map(|xname| xname.as_str().unwrap())
            })
            .collect(); */

        // Check user has provided valid XNAMES
        if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group).await {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }

        /* (included, excluded) =
            node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

        if !excluded.is_empty() {
            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}",
                    excluded,
                    hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                    hsm_groups_nodes);
            std::process::exit(-1);
        } */
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

    let client =
        get_k8s_client_programmatically(vault_base_url, vault_role_id, k8s_api_url).await?;

    let pods_fabric: Api<Pod> = Api::namespaced(client, "services");

    let params = kube::api::ListParams::default()
        .limit(1)
        .labels("app.kubernetes.io/name=cray-console-operator");

    let pods_objects = pods_fabric.list(&params).await?;

    let console_operator_pod = &pods_objects.items[0];
    let console_operator_pod_name = console_operator_pod.metadata.name.clone().unwrap();

    let mut attached = pods_fabric
        .exec(
            &console_operator_pod_name,
            vec!["sh", "-c", &format!("/app/get-node {}", xname)],
            &AttachParams::default()
                .container("cray-console-operator")
                .stderr(false),
        )
        .await?;

    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let next_stdout = stdout_stream.next().await.unwrap().unwrap();
    let stdout_str = std::str::from_utf8(&next_stdout).unwrap();
    let output_json: Value = serde_json::from_str(stdout_str)?;

    let console_pod_name = output_json["podname"].as_str().unwrap();

    let command = vec!["conman", "-j", xname];
    // let command = vec!["bash"];

    log::info!(
        "Alternatively run - kubectl -n services exec -it {} -c cray-console-node -- {}",
        console_pod_name,
        command
            .iter()
            .map(|x| (*x).to_string() + " ")
            .collect::<String>()
    );

    log::info!("Connecting to console {}", xname);

    let mut attached = pods_fabric
        .exec(
            console_pod_name,
            command,
            &AttachParams::default()
                .container("cray-console-node")
                .stdin(true)
                .stdout(true)
                .stderr(false) // Note to self: tty and stderr cannot both be true
                .tty(true),
        )
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
                            // match String::from_utf8_lossy(&remote_stdout) {
                            //     Ok(remote_stdout_str) => {
                            //         print!("{}", remote_stdout_str);
                            //         stdout.flush().unwrap();
                            //     },
                            //     Err(e) => {
                            //         // log::warn!("There was an error converting from utf8 to str:\n{:?}", &remote_stdout);
                            //         // log::error!("{:?}", e);
                            //         // stdout.suspend_raw_mode().unwrap();
                            //         // std::process::exit(1);
                            //     }
                            // }
                        }
                        Err(e) => {
                            log::warn!("There was an error reading stream input");
                            log::error!("{:?}", e);
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

    // let mut stdin_writer = attached.stdin().unwrap();
    // let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());

    // let stdin = std::io::stdin(); // We get `Stdin` here.
    // let rt = Runtime::new().unwrap();
    // rt.spawn(async move {
    //     let mut user_input = String::new();
    //     loop {
    //         stdin.read_line(&mut user_input).unwrap();
    //         stdin_writer.write_all(user_input.as_bytes()).await.unwrap();
    //     }
    // });

    // // let mut stdin = termion::async_stdin().keys();
    // // let rt = Runtime::new().unwrap();
    // // rt.spawn(async move {
    // //     loop {
    // //         let user_input = stdin.next();
    // //         if user_input.is_some() {
    // //             println!("Hi");
    // //             let user_input_b = user_input.unwrap().unwrap();
    // //             println!("{:?}", user_input_b);
    // //             stdin_writer.write_all(&vec![user_input_b, String::from("\n").as_bytes()[0]]).await.unwrap();
    // //         }
    // //     }
    // // });

    // let mut next_stdout;
    // let mut stdout_str;

    // loop {
    //     // Read command output
    //     next_stdout = stdout_stream.next().await;
    //     stdout_str = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
    //     print!("{}", stdout_str);
    //     std::io::stdout().flush().unwrap();
    // }
}

// async fn get_exec_command_output(mut attached: AttachedProcess) -> String {
//     let stdout = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
//     let out = stdout
//         .filter_map(|r| async { r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) })
//         .collect::<Vec<_>>()
//         .await
//         .join("");
//     attached.join().await.unwrap();
//     out
// }
