use backend_dispatcher::{interfaces::hsm::component::ComponentTrait, types::Group};
use mesa::{cfs, common::kubernetes};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{
        self, config_ops::K8sDetails, vault::http_client::fetch_shasta_k8s_secrets_from_vault,
    },
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    k8s_api_url: &str,
    group_available_vec: &[Group],
    user_input: &str,
    k8s: &K8sDetails,
) {
    let mut cfs_session_vec = cfs::session::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_or_else(|e| {
        eprintln!("ERROR - Could not get CFS sessions. Reason:\n{e}\nExit");
        std::process::exit(1);
    });

    let node_metadata_available_vec = backend
        .get_node_metadata_available(shasta_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
            std::process::exit(1);
        });

    // Convert user input to xname
    /* let xname_vec_rslt = common::node_ops::resolve_node_list_user_input_to_xname(
        backend,
        shasta_token,
        user_input,
        false,
        false,
    )
    .await; */
    let xname_vec_rslt = common::node_ops::resolve_node_list_user_input_to_xname_2(
        user_input,
        false,
        node_metadata_available_vec,
    )
    .await;

    // NOTE: fancy pattern matching. Maybe not a good use case for this. Ask in the future if this
    // is redeable or not
    let cfs_sessions_vec = match xname_vec_rslt.as_deref() {
        Ok([xname]) => {
            // Get most recent CFS session for node or group the node belongs to
            log::debug!("User input is a single node");
            cfs::session::utils::filter_by_xname(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_session_vec,
                &[xname],
                Some(&1),
            )
            .await;

            cfs_session_vec
        }
        Ok([_, ..]) => {
            // User input is an expression that expands to multiple nodes
            log::debug!("User input is a list of nodes");
            eprintln!("ERROR - Can only operate a single node. Exit");
            std::process::exit(1);
        }
        Ok([]) | Err(_) => {
            // Failed to convert user input to xname, try user input is either a group name or CFS session name
            log::debug!("User input is not a node. Checking user input as CFS session name");
            // Check if user input is a CFS session name
            let cfs_session_opt = cfs_session_vec
                .iter()
                .find(|session| session.name == Some(user_input.to_string()));

            if let Some(cfs_session) = cfs_session_opt {
                vec![cfs_session.clone()]
            } else if group_available_vec
                .iter()
                .map(|group| &group.label)
                .any(|group| group == user_input)
            {
                // Check if user input is a group name
                log::debug!("User input is not a node. Checking user input as group name");
                cfs::session::utils::filter_by_hsm(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &mut cfs_session_vec,
                    &[user_input.to_string()],
                    Some(&1),
                )
                .await;

                cfs_session_vec
            } else {
                // User input is neither a node, group name nor CFS session name
                eprintln!("ERROR - User input did not match node, group or session name. Exit");
                std::process::exit(1);
            }
        }
    };

    /* let session_name_opt = Some(&"".to_string());

    // Get CFS sessions
    let cfs_sessions_vec_opt = cfs::session::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
        None,
        Some(session_name),
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
    .await; */

    if cfs_sessions_vec.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    /* // FIXME: read this "validate_config_hsm_group_and_hsm_group_accessed" function and fix this
    // because we don't want calls directly to backend methods inside the client
    // Check HSM group in configurarion file can access CFS session
    let validation_rslt = hsm::group::utils::validate_config_hsm_group_and_hsm_group_accessed(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_config,
        session_name_opt,
        &cfs_sessions_vec,
    )
    .await;

    if let Err(e) = validation_rslt {
        eprintln!("ERROR - Validation failed. Reason:\n{e}\nExit");
        std::process::exit(1);
    }; */

    /* let shasta_k8s_secrets =
    fetch_shasta_k8s_secrets(vault_base_url, vault_secret_path, vault_role_id).await; */

    let shasta_k8s_secrets = match &k8s.authentication {
        common::config_ops::K8sAuth::Native {
            certificate_authority_data,
            client_certificate_data,
            client_key_data,
        } => {
            serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
        }
        common::config_ops::K8sAuth::Vault {
            base_url,
            secret_path,
            role_id,
        } => fetch_shasta_k8s_secrets_from_vault(&base_url, &secret_path, &role_id).await,
    };

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
