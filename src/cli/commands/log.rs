use core::time;
use std::error::Error;
use std::pin::Pin;
use std::thread;

use hyper::body::Bytes;
use k8s_openapi::api::core::v1::{Container, ContainerState, Pod};
use kube::Api;

use futures_util::{Stream, StreamExt, TryStreamExt};
use kube::api::ListParams;
use mesa::shasta::{cfs, hsm, kubernetes};
use tokio_stream::once;

use crate::common::vault::http_client::fetch_shasta_k8s_secrets;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    cluster_name: Option<&String>,
    session_name: Option<&String>,
    layer_id: Option<&u8>,
    hsm_group_config: Option<&String>,
) {
    // Get CFS sessions
    let cfs_sessions = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        cluster_name,
        session_name,
        None,
        None,
    )
    .await
    .unwrap();

    if cfs_sessions.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    // Check HSM group in configurarion file can access CFS session
    hsm::utils::validate_config_hsm_group_and_hsm_group_accessed(
        shasta_token,
        shasta_base_url,
        hsm_group_config,
        session_name,
        &cfs_sessions,
    )
    .await;

    let cfs_session_name: &str = cfs_sessions.last().unwrap()["name"].as_str().unwrap();

    let shasta_k8s_secrets = fetch_shasta_k8s_secrets(vault_base_url, vault_role_id).await;

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get CFS session logs
    let mut logs_stream = get_cfs_session_logs_stream(client, cfs_session_name, layer_id)
        .await
        .unwrap();

    while let Some(line) = logs_stream.try_next().await.unwrap() {
        print!("{}", std::str::from_utf8(&line).unwrap());
    }
}

pub async fn session_logs(
    vault_base_url: &str,
    vault_role_id: &str,
    cfs_session_name: &str,
    layer_id: Option<&u8>,
    k8s_api_url: &str,
) -> Result<
    Pin<Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>>,
    Box<dyn Error + Sync + Send>,
> {
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets(vault_base_url, vault_role_id).await;

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get CFS session logs
    get_cfs_session_logs_stream(client, cfs_session_name, layer_id).await
}

pub async fn get_container_logs_stream(
    cfs_session_layer_container: &Container,
    cfs_session_pod: &Pod,
    pods_api: &Api<Pod>,
    params: &ListParams,
) -> Result<
    Pin<Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>>,
    Box<dyn Error + Sync + Send>,
> {
    let mut container_log_stream: Pin<
        Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>,
    > = once(Ok(Bytes::copy_from_slice(
        format!(
            "\nFetching logs for container {}\n",
            cfs_session_layer_container.name
        )
        .as_bytes(),
    )))
    .boxed();

    // Check if container exists in pod
    let container_exists = cfs_session_pod
        .spec
        .as_ref()
        .unwrap()
        .containers
        .iter()
        .find(|x| x.name.eq(&cfs_session_layer_container.name));

    if container_exists.is_none() {
        return Err(format!(
            "Container {} does not exists. Aborting",
            cfs_session_layer_container.name
        )
        .into());
    }

    let mut container_state =
        get_container_state(cfs_session_pod, &cfs_session_layer_container.name);

    let mut i = 0;
    let max = 10;

    // Waiting for container ansible-x to start
    while container_state.as_ref().unwrap().waiting.is_some() && i <= max {
        container_log_stream = container_log_stream
            .chain(once(Ok(Bytes::copy_from_slice(
                format!(
                "\nWaiting for container {} to be ready. Checking again in 2 secs. Attempt {} of {}\n",
                cfs_session_layer_container.name,
                i + 1,
                max
            )
                .as_bytes(),
            ))))
            .boxed();

        /* print!(
            "\rWaiting for container {} to be ready. Checking again in 2 secs. Attempt {} of {}",
            ansible_container.name,
            i + 1,
            max
        ); */
        i += 1;
        thread::sleep(time::Duration::from_secs(2));
        let pods = pods_api.list(params).await?;
        container_state = get_container_state(&pods.items[0], &cfs_session_layer_container.name);
        log::debug!("Container state:\n{:#?}", container_state.as_ref().unwrap());
    }

    if container_state.as_ref().unwrap().waiting.is_some() {
        return Err(format!(
            "Container {} not ready. Aborting operation",
            cfs_session_layer_container.name
        )
        .into());
    }

    /* println!();
    println!("{green}***{color_reset} Starting logs for container {green}{container_name}{color_reset}", green = color::Fg(color::Green), container_name = ansible_container.name, color_reset = color::Fg(color::Reset));
    println!(); */

    /* let mut container_log_stream = once(Ok(Bytes::copy_from_slice(
        format!(
            "\n\n*** Starting logs for container {}\n\n",
            ansible_container.name
        )
        .as_bytes(),
    )))
    .boxed(); */

    let logs_stream = pods_api
        .log_stream(
            cfs_session_pod.metadata.name.as_ref().unwrap(),
            &kube::api::LogParams {
                follow: true,
                container: Some(cfs_session_layer_container.name.clone()),
                ..kube::api::LogParams::default()
            },
        )
        .await?;

    // We are going to use chain method (https://dtantsur.github.io/rust-openstack/tokio/stream/trait.StreamExt.html#method.chain) to join streams coming from kube_client::api::subresource::Api::log_stream which returns Result<impl Stream<Item = Result<Bytes>>> or Result<hyper::body::Bytes>, we will consume the Result hence we will be chaining streams of hyper::body::Bytes
    container_log_stream = container_log_stream.chain(logs_stream).boxed();

    Ok(container_log_stream)
}

pub async fn get_cfs_session_logs_stream(
    client: kube::Client,
    cfs_session_name: &str,
    layer_id: Option<&u8>,
) -> Result<
    Pin<Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>>,
    Box<dyn Error + Sync + Send>,
> {
    let mut container_log_stream: Pin<
        Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>,
    > = once(Ok(Bytes::copy_from_slice(
        format!("\nFetching logs for CFS session {}\n", cfs_session_name).as_bytes(),
    )))
    .boxed();

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
        container_log_stream = container_log_stream
            .chain(once(Ok(Bytes::copy_from_slice(
                format!(
                    "\nPod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of {}\n",
                    cfs_session_name,
                    i + 1,
                    max
                )
                .as_bytes(),
            ))))
            .boxed();
        /* print!(
            "\rPod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of {}",
            cfs_session_name,
            i + 1,
            max
        ); */
        i += 1;
        thread::sleep(time::Duration::from_secs(2));
        pods = pods_api.list(&params).await?;
    }

    if pods.items.is_empty() {
        return Err(format!(
            "Pod for cfs session {} not ready. Aborting operation",
            cfs_session_name
        )
        .into());
    }

    let cfs_session_pod = &pods.items[0].clone();

    let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
    log::info!("Pod name: {}", cfs_session_pod_name);

    let ansible_containers: Vec<&Container> = if layer_id.is_some() {
        // Printing a CFS session layer logs

        let layer = layer_id.unwrap().to_string();

        let container_name = format!("ansible-{}", layer);

        // Get ansible-x containers
        cfs_session_pod
            .spec
            .as_ref()
            .unwrap()
            .containers
            .iter()
            .filter(|container| container.name.eq(&container_name))
            .collect()
    } else {
        // Get ansible-x containers
        cfs_session_pod
            .spec
            .as_ref()
            .unwrap()
            .containers
            .iter()
            .filter(|container| container.name.contains("ansible-"))
            .collect()
    };

    for ansible_container in ansible_containers {
        container_log_stream = container_log_stream
            .chain(once(Ok(Bytes::copy_from_slice(
                format!(
                    "\n*** Starting logs for container {}\n",
                    ansible_container.name
                )
                .as_bytes(),
            ))))
            .boxed();

        let logs_stream =
            get_container_logs_stream(ansible_container, cfs_session_pod, &pods_api, &params)
                .await
                .unwrap();

        // We are going to use chain method (https://dtantsur.github.io/rust-openstack/tokio/stream/trait.StreamExt.html#method.chain) to join streams coming from kube_client::api::subresource::Api::log_stream which returns Result<impl Stream<Item = Result<Bytes>>> or Result<hyper::body::Bytes>, we will consume the Result hence we will be chaining streams of hyper::body::Bytes
        container_log_stream = container_log_stream.chain(logs_stream).boxed();
    }

    Ok(container_log_stream)
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
