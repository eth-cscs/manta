use std::collections::HashMap;

use mesa::{common::kubernetes, hsm};

use crate::{cli::commands::{config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all, power_on_nodes::is_user_input_nids}, common::{self, vault::http_client::fetch_shasta_k8s_secrets}};

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
    host_opt: Option<&String>,
) {
    let xname_opt = if let Some(host) = host_opt {
        // Get xname from nid
        let hsm_name_available_vec =
            get_hsm_name_without_system_wide_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await;

        // Get HSM group user has access to
        let hsm_group_available_map = mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_without_system_wide_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

        // Filter xnames to the ones members to HSM groups the user has access to

        // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
        let mut xname_vec = if is_user_input_nids(host) {
            log::debug!("User input seems to be NID");
            common::node_ops::nid_to_xname(
                shasta_base_url,
                shasta_token,
                shasta_root_cert,
                host,
                false,
            )
            .await
            .expect("Could not convert NID to XNAME")
        } else {
            log::debug!("User input seems to be XNAME");
            let hsm_group_summary: HashMap<String, Vec<String>> = 
                // Get HashMap with HSM groups and members curated for this request.
                // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
                // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
                // hostlist have been removed
                common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                    &host,
                hsm_group_available_map,
                false,
                )
                .await;

            hsm_group_summary.values().flatten().cloned().collect()
        };

        xname_vec.dedup();

        if xname_vec.is_empty() {
            eprintln!("ERROR - node '{}' not found", host);
        }

        log::debug!("input {} translates to xname {:?}", host, xname_vec);

        Some(xname_vec.first().unwrap().clone())
    } else {
        None
    };

    // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
    // `manta logs`

    // Get CFS sessions
    let cfs_sessions_vec_opt = mesa::cfs::session::mesa::http_client::get(
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

    // Filter CFS sessions by group or xname. If user input is xname, then find the most recent CFS session for that xname (keep in mind, this could mean the target for that session coul be the xname or the groups it belongs to)
    if let Some(xname) = xname_opt {
        mesa::cfs::session::mesa::utils::filter_by_xname(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_sessions_vec,
            &[&xname],
            Some(&1),
            true,
        )
        .await;
    } else {
        mesa::cfs::session::mesa::utils::filter_by_hsm(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_sessions_vec,
            hsm_name_vec,
            Some(&1),
            true
        )
        .await;
    }


    if cfs_sessions_vec.is_empty() {
        println!("No CFS session found");
        std::process::exit(0);
    }

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
