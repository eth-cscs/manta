use std::path::PathBuf;

use dialoguer::{theme::ColorfulTheme, Confirm};
use futures::TryStreamExt;
use mesa::{
    cfs::{self, session::csm::v3::r#struct::CfsSessionPostRequest},
    common::{jwt_ops, kubernetes, vault::http_client::fetch_shasta_k8s_secrets},
    error::Error,
    hsm,
    node::utils::validate_xnames_format_and_membership_agaisnt_single_hsm,
};

use k8s_openapi::chrono;
use substring::Substring;

use crate::common::local_git_repo;

/// Creates a CFS session target dynamic
/// Returns a tuple like (<cfs configuration name>, <cfs session name>)
pub async fn exec(
    gitea_token: &str,
    gitea_base_url: &str,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    k8s_api_url: &str,
    cfs_conf_sess_name: Option<&String>,
    playbook_yaml_file_name_opt: Option<&String>,
    hsm_group: Option<&String>,
    repos_paths: Vec<PathBuf>,
    ansible_limit: Option<String>,
    ansible_verbosity: Option<String>,
    ansible_passthrough: Option<String>,
    watch_logs: bool,
) -> (String, String) {
    /* let included: HashSet<String>;
    let excluded: HashSet<String>; */
    let mut xname_list: Vec<&str>;
    // Check andible limit matches the nodes in hsm_group
    let hsm_group_list;

    let cfs_configuration_name;

    let hsm_groups_node_list;

    // * Validate input params
    // Neither hsm_group (both config file or cli arg) nor ansible_limit provided --> ERROR since we don't know the target nodes to apply the session to
    // NOTE: hsm group can be assigned either by config file or cli arg
    if ansible_limit.is_none() && hsm_group.is_none() && hsm_group.is_none() {
        // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
        eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
        std::process::exit(1);
    }

    // * End validation input params

    // * Parse input params
    // Parse ansible limit
    // Get ansible limit nodes from cli arg
    let ansible_limit = ansible_limit.unwrap_or_default();
    let ansible_limit_nodes: Vec<&str> =
        ansible_limit.split(',').map(|xname| xname.trim()).collect();

    // Parse hsm group
    let mut hsm_group_value_opt = None;

    // Get hsm_group from cli arg
    if hsm_group.is_some() {
        hsm_group_value_opt = hsm_group;
    }
    // * End Parse input params

    cfs_configuration_name = cfs_conf_sess_name.unwrap();

    // * Process/validate hsm group value (and ansible limit)
    if let Some(hsm_group_value) = hsm_group_value_opt {
        // Get all hsm groups details related to hsm_group input
        hsm_group_list = crate::common::cluster_ops::get_details(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_value,
        )
        .await;

        // Take all nodes for all hsm_groups found and put them in a Vec
        hsm_groups_node_list = hsm_group_list
            .iter()
            .flat_map(|hsm_group| hsm_group.members.iter().map(|xname| xname.as_str()))
            .collect();

        if !ansible_limit_nodes.is_empty() {
            // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group
            xname_list = hsm_groups_node_list;
            // Check user has provided valid XNAMES
            if !validate_xnames_format_and_membership_agaisnt_single_hsm(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &xname_list,
                hsm_group,
            )
            .await
            {
                eprintln!("xname/s invalid. Exit");
                std::process::exit(1);
            }
        } else {
            // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
            // included = hsm_groups_nodes
            xname_list = hsm_groups_node_list;
        }
    } else {
        // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
        // included = ansible_limit_nodes
        xname_list = ansible_limit_nodes;
    }

    // * End Process/validate hsm group value (and ansible limit)

    // Remove duplicates in xname_list
    xname_list.sort();
    xname_list.dedup();

    log::info!("Replacing '_' with '-' in repo name.");
    let cfs_configuration_name = str::replace(&cfs_configuration_name, "_", "-");

    // * Check nodes are ready to run, create CFS configuration and CFS session
    let cfs_session_name = check_nodes_are_ready_to_run_cfs_configuration_and_run_cfs_session(
        &cfs_configuration_name,
        playbook_yaml_file_name_opt,
        repos_paths,
        gitea_token,
        gitea_base_url,
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(xname_list.into_iter().collect::<Vec<_>>().join(",")), // Convert Hashset to String with comma separator, need to convert to Vec first following https://stackoverflow.com/a/47582249/1918003
        Some(
            ansible_verbosity
                .unwrap_or("3".to_string())
                .parse::<u8>()
                .unwrap_or(0),
        ),
        ansible_passthrough,
    )
    .await
    .unwrap();

    // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
    // `manta logs`
    if watch_logs {
        log::info!("Fetching logs ...");
        /* let mut logs_stream = cli::commands::log::get_cfs_session_container_ansible_logs_stream(
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            &cfs_session.name,
            None,
            k8s_api_url,
        )
        .await
        .unwrap(); */

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
            kubernetes::get_cfs_session_container_ansible_logs_stream(client, &cfs_session_name)
                .await
                .unwrap();

        while let Some(line) = logs_stream.try_next().await.unwrap() {
            println!("{}", line);
        }
    }
    // * End Create CFS session

    // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply session", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap());

    (cfs_configuration_name, cfs_session_name)
}

pub async fn check_nodes_are_ready_to_run_cfs_configuration_and_run_cfs_session(
    cfs_configuration_name: &str,
    playbook_yaml_file_name_opt: Option<&String>,
    repos: Vec<PathBuf>,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    limit: Option<String>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<String>,
) -> Result<String, Error> {
    // Get ALL sessions
    let cfs_sessions = cfs::session::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    // FIXME: things to fix:
    //  - extend the list of nodes checked being modified byt also including those in CFS sessions
    //  working against HSM groups (and not just 'ansible limit')
    //  - get also the list of CFS sessions affecting those nodes
    let nodes_in_running_or_pending_cfs_session: Vec<&str> = cfs_sessions
        .iter()
        .filter(|cfs_session| {
            ["running", "pending"].contains(
                &cfs_session
                    .status
                    .as_ref()
                    .unwrap()
                    .session
                    .as_ref()
                    .unwrap()
                    .status
                    .as_ref()
                    .unwrap()
                    .as_str(),
            ) && cfs_session
                .configuration
                .as_ref()
                .unwrap()
                .name
                .as_ref()
                .unwrap()
                == cfs_configuration_name
        })
        .flat_map(|cfs_session| {
            cfs_session
                .ansible
                .as_ref()
                .unwrap()
                .limit
                .as_ref()
                .unwrap()
                .split(',')
        })
        .map(|xname| xname.trim())
        .collect(); // TODO: remove duplicates... sort() + dedup() ???

    log::info!(
        "Nodes with cfs session running or pending: {:?}",
        nodes_in_running_or_pending_cfs_session
    );

    // NOTE: nodes can be a list of xnames or hsm group name

    // Convert limit (String with list of target nodes for new CFS session) into list of String
    let limit_value = limit.clone().unwrap_or("".to_string());
    let nodes_list: Vec<&str> = limit_value.split(',').map(|node| node.trim()).collect();

    // Check each node if it has a CFS session already running
    for node in nodes_list {
        if nodes_in_running_or_pending_cfs_session.contains(&node) {
            eprintln!(
                "The node '{}' from the list provided is already assigned to a running/pending CFS session. Please try again latter or delete the CFS session. Exitting", node
            );
            std::process::exit(1);
        }
    }

    // Check nodes are ready to run a CFS layer
    let xnames: Vec<String> = limit_value
        .split(',')
        .map(|xname| String::from(xname.trim()))
        .collect();

    for xname in xnames {
        log::info!("Checking status of component {}", xname);

        let component_status = cfs::component::csm::v2::get_single_component(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &xname,
        )
        .await?;

        let hsm_component_status_rslt = hsm::component_status::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &[xname.clone()],
        )
        .await?;

        let hsm_component_status_state = &hsm_component_status_rslt
            // ["Components"]
            // .as_array()
            // .unwrap()
            .first()
            .unwrap()["State"];

        log::info!(
            "HSM component state for component {}: {}",
            xname,
            hsm_component_status_state.as_str().unwrap()
        );
        log::info!(
            "Is component enabled for batched CFS: {}",
            component_status["enabled"]
        );
        log::info!("Error count: {}", component_status["errorCount"]);

        if hsm_component_status_state.eq("On") || hsm_component_status_state.eq("Standby") {
            log::info!("There is an CFS session scheduled to run on this node. Pleas try again later. Aborting");
            std::process::exit(0);
        }
    }

    // Check local repos
    let mut layers_summary = vec![];

    for (i, repo_path) in repos.iter().enumerate() {
        // log::debug!("Local repo: {} state: {:?}", repo.path().display(), repo.state());
        // TODO: check each folder has a real git repo
        // TODO: check each folder has expected file name manta/shasta expects to find the main ansible playbook
        // TODO: a repo could param value could be a repo name, a filesystem path pointing to a repo or an url pointing to a git repo???
        // TODO: format logging on screen so it is more readable

        // Get repo from path
        let repo = match local_git_repo::get_repo(&repo_path.to_string_lossy()) {
            Ok(repo) => repo,
            Err(_) => {
                eprintln!(
                    "Could not find a git repo in {}",
                    repos[i].to_string_lossy()
                );
                std::process::exit(1);
            }
        };

        // Get last (most recent) commit
        let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

        log::info!("Checking local repo status ({})", &repo.path().display());

        // Check if all changes in local repo has been commited locally
        if !local_git_repo::untracked_changed_local_files(&repo).unwrap() {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Your local repo has uncommitted changes. Do you want to continue?")
                .interact()
                .unwrap()
            {
                println!(
                    "Continue. Checking commit id {} against remote",
                    local_last_commit.id()
                );
            } else {
                println!("Cancelled by user. Aborting.");
                std::process::exit(0);
            }
        }

        /* // Check site.yml file exists inside repo folder
        if !Path::new(repo.path()).exists() {
            eprintln!(
                "site.yaml or 'playbook' file does not exists in {}",
                repo.path().display()
            );
            std::process::exit(1);
        } */

        // Get repo name
        let repo_ref_origin = repo.find_remote("origin").unwrap();

        log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());

        let repo_ref_origin_url = repo_ref_origin.url().unwrap();

        let repo_name = repo_ref_origin_url.substring(
            repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
            repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
        );

        let timestamp = local_last_commit.time().seconds();
        let tm = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();

        log::debug!("\n\nCommit details to apply to CFS layer:\nCommit  {}\nAuthor: {}\nDate:   {}\n\n    {}\n", local_last_commit.id(), local_last_commit.author(), tm, local_last_commit.message().unwrap_or("no commit message"));

        let layer_summary = vec![
            i.to_string(),
            repo_name.to_string(),
            local_git_repo::untracked_changed_local_files(&repo)
                .unwrap()
                .to_string(),
        ];

        layers_summary.push(layer_summary);
    }

    // Print CFS session/configuration layers summary on screen
    println!("Please review the following CFS layers:",);
    for layer_summary in layers_summary {
        println!(
            " - Layer-{}; repo name: {}; local changes committed: {}",
            layer_summary[0], layer_summary[1], layer_summary[2]
        );
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Please review the layers and its order and confirm if proceed. Do you want to continue?")
        .interact()
        .unwrap()
    {
        println!("Continue. Creating new CFS configuration and layer(s)");
    } else {
        println!("Cancelled by user. Aborting.");
        std::process::exit(0);
    }

    log::info!("Creating CFS configuration {}", cfs_configuration_name);

    let cfs_configuration = crate::common::cfs_configuration_utils::create_from_repos(
        gitea_token,
        gitea_base_url,
        shasta_root_cert,
        repos,
        playbook_yaml_file_name_opt,
    )
    .await;

    // Update/PUT CFS configuration
    let cfs_configuration_resp = cfs::configuration::put(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration,
        cfs_configuration_name,
    )
    .await;

    let cfs_configuration_name = match cfs_configuration_resp {
        Ok(_) => &cfs_configuration_resp.as_ref().unwrap().name,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Create dynamic CFS session
    let cfs_session_name = format!(
        "{}-{}",
        cfs_configuration_name,
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );

    let session = CfsSessionPostRequest::new(
        cfs_session_name,
        cfs_configuration_name.clone(),
        None,
        limit,
        None,
        ansible_verbosity,
        ansible_passthrough,
        false,
        None,
        None,
        None,
        false,
        None,
    );

    let cfs_session_resp =
        cfs::session::post(shasta_token, shasta_base_url, shasta_root_cert, &session).await;

    let cfs_session_name = match cfs_session_resp {
        Ok(_) => cfs_session_resp.as_ref().unwrap().name.as_ref().unwrap(),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    Ok(String::from(cfs_session_name))
}
