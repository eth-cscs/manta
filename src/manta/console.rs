use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, AttachedProcess},
    Api,
};
use serde_json::Value;
use tokio_util::io::ReaderStream;

use crate::{
    common::vault::http_client::fetch_shasta_k8s_secrets,
    shasta::kubernetes::get_k8s_client_programmatically,
};

pub async fn get_container_attachment(
    xname: &String,
    vault_base_url: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
) -> AttachedProcess {
    log::info!("xname: {}", xname);

    let shasta_k8s_secrets = fetch_shasta_k8s_secrets(vault_base_url, vault_role_id).await;

    let client = get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    let pods_fabric: Api<Pod> = Api::namespaced(client, "services");

    let params = kube::api::ListParams::default()
        .limit(1)
        .labels("app.kubernetes.io/name=cray-console-operator");

    let pods_objects = pods_fabric.list(&params).await.unwrap();

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
        .await
        .unwrap();

    let mut stdout_stream = ReaderStream::new(attached.stdout().unwrap());
    let next_stdout = stdout_stream.next().await.unwrap().unwrap();
    let stdout_str = std::str::from_utf8(&next_stdout).unwrap();
    let output_json: Value = serde_json::from_str(stdout_str).unwrap();

    let console_pod_name = output_json["podname"].as_str().unwrap();

    let command = vec!["conman", "-j", xname]; // Enter the container and open conman to access node's console
                                               // let command = vec!["bash"]; // Enter the container and open bash to start an interactive
                                               // terminal session

    log::info!(
        "Alternatively run - kubectl -n services exec -it {} -c cray-console-node -- {}",
        console_pod_name,
        command
            .iter()
            .map(|x| (*x).to_string() + " ")
            .collect::<String>()
    );

    log::info!("Connecting to console {}", xname);

    pods_fabric
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
        .await
        .unwrap()
}
