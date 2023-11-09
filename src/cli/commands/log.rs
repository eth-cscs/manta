use std::error::Error;

use futures::{io::Lines, AsyncBufReadExt, TryStreamExt};

use mesa::shasta::{cfs, hsm, kubernetes};

use crate::common::vault::http_client::fetch_shasta_k8s_secrets;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    cluster_name: Option<&String>,
    session_name: Option<&String>,
    // layer_id: Option<&u8>,
    hsm_group_config: Option<&String>,
) {
    // Get CFS sessions
    let cfs_sessions_resp = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        cluster_name,
        session_name,
        None,
        None,
    )
    .await
    .unwrap();

    if cfs_sessions_resp.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    // Check HSM group in configurarion file can access CFS session
    hsm::utils::validate_config_hsm_group_and_hsm_group_accessed(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_config,
        session_name,
        &cfs_sessions_resp,
    )
    .await;

    let cfs_session_name: &str = cfs_sessions_resp.last().unwrap()["name"].as_str().unwrap();

    let shasta_k8s_secrets =
        fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get CFS session logs
    let logs_stream_rslt = kubernetes::get_cfs_session_container_git_clone_logs_stream(
        client.clone(),
        &cfs_session_name,
    )
    .await;

    match logs_stream_rslt {
        Ok(mut logs_stream) => {
            while let Some(line) = logs_stream.try_next().await.unwrap() {
                println!("{}", line);
            }
        }
        Err(error_msg) => log::error!("{}", error_msg),
    }

    let mut logs_stream =
        kubernetes::get_cfs_session_container_ansible_logs_stream(client, cfs_session_name)
            .await
            .unwrap();

    while let Some(line) = logs_stream.try_next().await.unwrap() {
        println!("{}", line);
    }
}

/* pub async fn get_cfs_session_container_ansible_logs_stream(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    cfs_session_name: &str,
    k8s_api_url: &str,
) -> Result<Lines<impl AsyncBufReadExt>, Box<dyn Error + Sync + Send>> {
    let shasta_k8s_secrets =
        fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get CFS session logs
    kubernetes::get_cfs_session_container_ansible_logs_stream(client, cfs_session_name).await
} */
