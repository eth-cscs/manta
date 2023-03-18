use core::time;
use std::{error::Error, thread};

use k8s_openapi::api::core::v1::{ContainerState, Pod};
use kube::Api;

use futures_util::{StreamExt, TryStreamExt};
use termion::color;

use crate::shasta_cfs_session;

use crate::shasta::kubernetes as shasta_k8s;

use clap::ArgMatches;

pub async fn exec(
    cli_log: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &String,
) {
    let logging_session_name = cli_log.get_one::<String>("SESSION");

    let layer_id = cli_log.get_one::<u8>("layer-id");

    session_logs_proxy(
        shasta_token,
        shasta_base_url,
        vault_base_url,
        vault_role_id,
        None,
        logging_session_name,
        layer_id,
    )
    .await
    .unwrap();
}

pub async fn session_logs_proxy(
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &String,
    cluster_name: Option<&String>,
    session_name: Option<&String>,
    layer_id: Option<&u8>,
) -> Result<(), Box<dyn Error>> {
    // Get CFS sessions
    let cfs_sessions = shasta_cfs_session::http_client::get(
        shasta_token,
        shasta_base_url,
        cluster_name,
        session_name,
        None,
        None,
    )
    .await?;

    if cfs_sessions.is_empty() {
        log::info!("No CFS session found");
        return Ok(());
    }

    let cfs_session_name: &str = cfs_sessions.last().unwrap()["name"].as_str().unwrap();

    let client = shasta_k8s::get_k8s_client_programmatically(vault_base_url, vault_role_id).await?;

    // Get CFS session logs
    get_container_logs(client, cfs_session_name, layer_id).await?;

    Ok(())
}

pub async fn session_logs(
    vault_base_url: &str,
    vault_role_id: &String,
    cfs_session_name: &str,
    layer_id: Option<&u8>,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
    let client = shasta_k8s::get_k8s_client_programmatically(vault_base_url, vault_role_id).await?;

    // Get CFS session logs
    get_container_logs(client, cfs_session_name, layer_id).await?;

    Ok(())
}

fn get_container_state(pod: &Pod, container_name: &String) -> Option<ContainerState> {
    let container_status = pod
        .status
        .as_ref()
        .unwrap()
        .container_statuses
        .as_ref()
        .unwrap()
        .iter()
        .find(|container_status| container_status.name.eq(container_name));

    match container_status {
        Some(container_status_aux) => container_status_aux.state.clone(),
        None => None,
    }
}

pub async fn get_container_logs(
    client: kube::Client,
    cfs_session_name: &str,
    layer_id: Option<&u8>,
) -> Result<(), Box<dyn Error>> {
    let pods_api: Api<Pod> = Api::namespaced(client, "services");

    log::debug!("cfs session: {}", cfs_session_name);

    let params = kube::api::ListParams::default()
        .limit(1)
        .labels(format!("cfsession={}", cfs_session_name).as_str());

    let mut pods = pods_api.list(&params).await?;

    let mut i = 0;
    let max = 10;
    // Waiting for pod to start
    while pods.items.is_empty() && i <= max {
        print!(
            "\rPod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of {}",
            cfs_session_name,
            i + 1,
            max
        );
        i += 1;
        thread::sleep(time::Duration::from_secs(2));
        pods = pods_api.list(&params).await?;
    }

    println!();

    if pods.items.is_empty() {
        eprintln!(
            "Pod for cfs session {} not ready. Aborting operation",
            cfs_session_name
        );
        std::process::exit(1);
    }

    let cfs_session_pod = &pods.items[0].clone();

    let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
    log::info!("Pod name: {}", cfs_session_pod_name);

    if layer_id.is_some() {
        // Printing a CFS session layer logs

        let layer = layer_id.unwrap().to_string();

        let container_name = format!("ansible-{}", layer);

        // Check if container exists in pod
        let container_exists = cfs_session_pod
            .spec
            .as_ref()
            .unwrap()
            .containers
            .iter()
            .find(|x| x.name.eq(&container_name));

        if container_exists.is_none() {
            println!(
                "Container {} (layer {}) does not exists. Aborting",
                container_name, layer
            );
            std::process::exit(0);
        }

        let mut container_state = get_container_state(cfs_session_pod, &container_name);

        let mut i = 0;
        let max = 10;

        // Waiting for container ansible-x to start
        while container_state.as_ref().unwrap().waiting.is_some() && i <= max {
            print!(
                "\rWaiting for container {} to be ready. Checking again in 2 seconds. Attempt {} of {}",
                container_name,
                i + 1, max
            );
            i += 1;
            thread::sleep(time::Duration::from_secs(2));
            pods = pods_api.list(&params).await?;
            container_state = get_container_state(&pods.items[0], &container_name);
            log::debug!("Container state:\n{:#?}", container_state.as_ref().unwrap());
        }

        println!();

        if container_state.as_ref().unwrap().waiting.is_some() {
            eprintln!("Container {} not ready. Aborting operation", container_name);
            std::process::exit(1);
        }

        let mut logs = pods_api
            .log_stream(
                &cfs_session_pod_name,
                &kube::api::LogParams {
                    follow: true,
                    // tail_lines: Some(1),
                    container: Some(container_name),
                    ..kube::api::LogParams::default()
                },
            )
            .await?
            .boxed();

        while let Some(line) = logs.try_next().await? {
            print!("{}", std::str::from_utf8(&line).unwrap());
        }
    } else {
        // Printing logs of all CFS layers

        // Get ansible-x containers
        let ansible_containers = cfs_session_pod
            .spec
            .as_ref()
            .unwrap()
            .containers
            .iter()
            .filter(|x| x.name.contains("ansible-"));

        for ansible_container in ansible_containers {
            println!();
            println!("{green}***{color_reset} Starting logs for container {green}{container_name}{color_reset}", green = color::Fg(color::Green), container_name = ansible_container.name, color_reset = color::Fg(color::Reset));
            println!();

            // Check if container exists in pod
            let container_exists = cfs_session_pod
                .spec
                .as_ref()
                .unwrap()
                .containers
                .iter()
                .find(|x| x.name.eq(&ansible_container.name));

            if container_exists.is_none() {
                println!(
                    "Container {} does not exists. Aborting",
                    ansible_container.name
                );
                std::process::exit(0);
            }

            let mut container_state = get_container_state(cfs_session_pod, &ansible_container.name);

            let mut i = 0;
            let max = 10;
            // Waiting for container ansible-x to start
            while container_state.as_ref().unwrap().waiting.is_some() && i <= max {
                print!("\rWaiting for container {} to be ready. Checking again in 2 secs. Attempt {} of {}", ansible_container.name, i + 1, max);
                i += 1;
                thread::sleep(time::Duration::from_secs(2));
                pods = pods_api.list(&params).await?;
                container_state = get_container_state(&pods.items[0], &ansible_container.name);
                log::debug!("Container state:\n{:#?}", container_state.as_ref().unwrap());
            }

            println!();

            if container_state.as_ref().unwrap().waiting.is_some() {
                eprintln!(
                    "Container {} not ready. Aborting operation",
                    ansible_container.name
                );
                std::process::exit(1);
            }

            let mut logs = pods_api
                .log_stream(
                    &cfs_session_pod_name,
                    &kube::api::LogParams {
                        follow: true,
                        // tail_lines: Some(1),
                        container: Some(ansible_container.name.clone()),
                        ..kube::api::LogParams::default()
                    },
                )
                .await?
                .boxed();

            while let Some(line) = logs.try_next().await? {
                print!("{}", std::str::from_utf8(&line).unwrap());
            }
        }
    }

    Ok(())
}
