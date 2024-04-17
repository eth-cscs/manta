use mesa::{common::kubernetes, hsm};

use crate::common::{self, vault::http_client::fetch_shasta_k8s_secrets};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    hsm_name_vec: &[String],
    session_name_opt: Option<&String>,
    hsm_group_config: Option<&String>,
) {
    // Get CFS sessions
    let cfs_sessions_resp_opt = mesa::cfs::session::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
        None,
        session_name_opt,
        None,
    )
    .await;

    let mut cfs_sessions_resp = match cfs_sessions_resp_opt {
        Ok(cfs_sessions_resp) => cfs_sessions_resp,
        Err(error) => {
            eprintln!(
                "ERROR: CFS session '{}' not found.\nReason: {:#?}\nExit",
                session_name_opt.unwrap(),
                error
            );
            std::process::exit(1);
        }
    };

    mesa::cfs::session::mesa::utils::filter_by_hsm(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_sessions_resp,
        hsm_name_vec,
        Some(&1),
    )
    .await;

    if cfs_sessions_resp.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    // Check HSM group in configurarion file can access CFS session
    hsm::group::mesa::utils::validate_config_hsm_group_and_hsm_group_accessed(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_config,
        session_name_opt,
        &cfs_sessions_resp,
    )
    .await;

    let shasta_k8s_secrets =
        fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

    log::info!(
        "Get logs for CFS session:\n{}",
        common::cfs_session_utils::get_table_struct(&cfs_sessions_resp)
    );

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    kubernetes::print_cfs_session_logs(
        client,
        cfs_sessions_resp.first().unwrap().name.as_ref().unwrap(),
    )
    .await;
}
