use std::error::Error;
use std::pin::Pin;

use futures_util::{Stream, TryStreamExt};
use mesa::shasta::{cfs, hsm, kubernetes::{self, get_cfs_session_logs_stream}};

use crate::common::vault::http_client::fetch_shasta_k8s_secrets;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_secret_path: &str,
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

    let shasta_k8s_secrets = fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

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
    vault_secret_path: &str,
    vault_role_id: &str,
    cfs_session_name: &str,
    layer_id: Option<&u8>,
    k8s_api_url: &str,
) -> Result<
    Pin<Box<dyn Stream<Item = Result<hyper::body::Bytes, kube::Error>> + std::marker::Send>>,
    Box<dyn Error + Sync + Send>,
> {
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get CFS session logs
    get_cfs_session_logs_stream(client, cfs_session_name, layer_id).await
}
