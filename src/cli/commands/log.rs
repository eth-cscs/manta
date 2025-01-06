use mesa::{cfs, common::kubernetes, hsm};

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
    // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
    // `manta logs`

    // Get CFS sessions
    let cfs_sessions_vec_opt = cfs::session::get(
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

    let mut cfs_sessions_vec = match cfs_sessions_vec_opt {
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

    cfs::session::utils::filter_by_hsm(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_sessions_vec,
        hsm_name_vec,
        Some(&1),
    )
    .await;

    if cfs_sessions_vec.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    // FIXME: read this "validate_config_hsm_group_and_hsm_group_accessed" function and fix this
    // because we don't want calls directly to backend methods inside the client
    // Check HSM group in configurarion file can access CFS session
    hsm::group::utils::validate_config_hsm_group_and_hsm_group_accessed(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_config,
        session_name_opt,
        &cfs_sessions_vec,
    )
    .await;

    let shasta_k8s_secrets =
        fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await;

    log::info!(
        "Get logs for CFS session:\n{}",
        common::cfs_session_utils::get_table_struct(&cfs_sessions_vec)
    );

    let client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    let log_rslt = kubernetes::print_cfs_session_logs(
        client,
        cfs_sessions_vec.first().unwrap().name.as_ref().unwrap(),
    )
    .await;

    if let Err(e) = log_rslt {
        eprintln!("ERROR - {e}. Exit");
        std::process::exit(1);
    }
}
