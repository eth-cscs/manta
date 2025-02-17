use backend_dispatcher::{
    interfaces::{cfs::CfsTrait, hsm::component::ComponentTrait},
    types::Group,
};

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
    let node_metadata_available_vec = backend
        .get_node_metadata_available(shasta_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
            std::process::exit(1);
        });

    let xname_vec_rslt = common::node_ops::resolve_node_list_user_input_to_xname_2(
        user_input,
        false,
        node_metadata_available_vec,
    )
    .await;

    /* let mut cfs_session_vec: Vec<
        mesa::cfs::session::http_client::v3::types::CfsSessionGetResponse,
    > = mesa::cfs::session::http_client::v3::get(
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
    }); */

    // NOTE: fancy pattern matching. Maybe not a good use case for this. Ask in the future if this
    // is redeable or not
    let cfs_sessions_vec_rslt = match xname_vec_rslt.as_deref() {
        Ok([]) | Err(_) => {
            // Failed to convert user input to xname, try user input is either a group name or CFS session name
            log::debug!("User input is not a node. Checking user input as CFS session name");
            // Check if user input is a CFS session name
            backend
                .get_sessions(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&user_input.to_string()),
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

            /* let cfs_session_opt = cfs_session_vec
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
                mesa::cfs::session::utils::filter_by_hsm(
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
            } */
        }
        Ok([xname]) => {
            // Get most recent CFS session for node or group the node belongs to
            log::debug!("User input is a single node");
            /* mesa::cfs::session::utils::filter_by_xname(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_session_vec,
                &[xname],
                Some(&1),
            )
            .await; */
            backend
                .get_sessions_by_xname(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &[xname],
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
        }
        Ok([_, ..]) => {
            // User input is an expression that expands to multiple nodes
            log::debug!("User input is a list of nodes");
            eprintln!("ERROR - Can only operate a single node. Exit");
            std::process::exit(1);
        }
    };

    let cfs_sessions_vec = cfs_sessions_vec_rslt.unwrap_or_else(|e| {
        eprintln!("ERROR - Could not get CFS sessions. Reason:\n{e}\nExit");
        std::process::exit(1);
    });

    if cfs_sessions_vec.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

    log::info!(
        "Get logs for CFS session:\n{}",
        common::cfs_session_utils::get_table_struct(&cfs_sessions_vec)
    );

    // Get K8s secrets
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

    let client =
        mesa::common::kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
            .await
            .unwrap();

    let log_rslt = mesa::common::kubernetes::print_cfs_session_logs(
        client,
        cfs_sessions_vec.first().unwrap().name.as_ref().unwrap(),
    )
    .await;

    if let Err(e) = log_rslt {
        eprintln!("ERROR - {e}. Exit");
        std::process::exit(1);
    }
}
