use backend_dispatcher::{
    error::Error,
    interfaces::{cfs::CfsTrait, hsm::component::ComponentTrait},
    types::{Group, K8sDetails},
};
use mesa::cfs::{self, session::http_client::v3::types::CfsSessionGetResponse};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self},
};

use futures::{AsyncBufReadExt, TryStreamExt};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    site_name: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
        }
        Ok([xname]) => {
            // Get most recent CFS session for node or group the node belongs to
            log::debug!("User input is a single node");

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

    // FIXME: at this point we are calling a mesa function with CFS sessions from backend. We
    // need to convert them to mesa CFS sessions
    // FIXME: convert from CFS session backend to mesa CFS session
    let cfs_session = cfs_sessions_vec.first().unwrap();

    // FIXME: convert from CFS session backend to mesa CFS session
    let cfs_session_backend: CfsSessionGetResponse = cfs_session.clone().into();

    let group_available_vec: Vec<mesa::hsm::group::types::Group> = group_available_vec
        .into_iter()
        .map(|group| group.clone().into())
        .collect::<Vec<_>>();

    cfs::session::utils::check_cfs_session_against_groups_available(
        &cfs_session_backend,
        group_available_vec,
    );

    // Get K8s secrets
    /* let shasta_k8s_secrets = match &k8s.authentication {
        K8sAuth::Native {
            certificate_authority_data,
            client_certificate_data,
            client_key_data,
        } => {
            serde_json::json!(
            {
                "certificate-authority-data": certificate_authority_data,
                "client-certificate-data": client_certificate_data,
                "client-key-data": client_key_data
            })
        }
        K8sAuth::Vault {
            base_url,
            // secret_path,
            // role_id,
        } => fetch_shasta_k8s_secrets_from_vault(&base_url, site_name, shasta_token)
            .await
            .unwrap(),
    };

    let client =
        mesa::common::kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
            .await
            .unwrap();

    let log_rslt = mesa::common::kubernetes::print_cfs_session_logs(
        client,
        cfs_session.name.as_ref().unwrap(),
    )
    .await; */

    let log_rslt = print_cfs_session_logs(
        backend,
        shasta_token,
        site_name,
        cfs_session.name.as_ref().unwrap(),
        k8s,
    )
    .await;

    if let Err(e) = log_rslt {
        eprintln!("ERROR - {e}. Exit");
        std::process::exit(1);
    }
}

pub async fn print_cfs_session_logs(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    site_name: &str,
    cfs_session_name: &str,
    k8s: &K8sDetails,
) -> Result<(), Error> {
    let logs_stream = backend
        .get_session_logs_stream(shasta_token, site_name, cfs_session_name, k8s)
        .await?;

    /* get_cfs_session_init_container_git_clone_logs_stream(client.clone(), cfs_session_name)
    .await?; */

    let mut lines = logs_stream.lines();

    while let Some(line) = lines.try_next().await.unwrap() {
        println!("{}", line);
    }

    Ok(())
}
