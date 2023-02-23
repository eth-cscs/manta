use std::collections::HashSet;

use clap::ArgMatches;

use std::path::Path;

use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::shasta::cfs::configuration;
use crate::shasta::cfs::session::http_client;
use crate::shasta::hsm;
use crate::{shasta_cfs_component, shasta_cfs_session};
use k8s_openapi::chrono;
use serde_json::Value;
use substring::Substring;

use crate::common::{gitea, local_git_repo};

pub async fn exec(
    gitea_token: &str,
    gitea_base_url: &str,
    vault_base_url: String,
    hsm_group: Option<&String>,
    cli_apply_session: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) -> () {
    let included: HashSet<String>;
    let excluded: HashSet<String>;
    // Check andible limit matches the nodes in hsm_group
    let hsm_groups;

    let cfs_configuration_name;

    let hsm_groups_nodes;

    // * Validate input params
    // Neither hsm_group (both config file or cli arg) nor ansible_limit provided --> ERROR since we don't know the target nodes to apply the session to
    // NOTE: hsm group can be assigned either by config file or cli arg
    if cli_apply_session
        .get_one::<String>("ansible-limit")
        .is_none()
        && hsm_group.is_none()
        && cli_apply_session.get_one::<String>("hsm-group").is_none()
    {
        // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
        eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
        std::process::exit(-1);
    }
    // * End validation input params

    // * Parse input params
    // Parse ansible limit
    // Get ansible limit nodes from cli arg
    let ansible_limit_nodes: HashSet<String> = if cli_apply_session
        .get_one::<String>("ansible-limit")
        .is_some()
    {
        // Get HashSet with all nodes from ansible-limit param
        cli_apply_session
            .get_one::<String>("ansible-limit")
            .unwrap()
            .replace(' ', "") // trim xnames by removing white spaces
            .split(',')
            .map(|xname| xname.to_string())
            .collect()
    } else {
        HashSet::new()
    };

    // Parse hsm group
    let mut hsm_group_value = None;

    // Get hsm_group from cli arg
    if cli_apply_session.get_one::<String>("hsm-group").is_some() {
        hsm_group_value = cli_apply_session.get_one::<String>("hsm-group");
    }

    // Get hsm group from config file
    if hsm_group.is_some() {
        hsm_group_value = hsm_group;
    }
    // * End Parse input params

    // * Process/validate hsm group value (and ansible limit)
    if hsm_group_value.is_some() {
        // Get all hsm groups related to hsm_group input
        hsm_groups =
            crate::common::cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group_value.unwrap())
                .await;

        //cfs_configuration_name = format!("{}-{}", hsm_group_value.unwrap(), cli_apply_session.get_one::<String>("name").unwrap());
        cfs_configuration_name = cli_apply_session
            .get_one::<String>("name")
            .unwrap()
            .to_string();

        // Take all nodes for all hsm_groups found and put them in a Vec
        hsm_groups_nodes = hsm_groups
            .iter()
            .flat_map(|hsm_group| {
                hsm_group
                    .members
                    .iter()
                    .map(|xname| xname.as_str().unwrap().to_string())
            })
            .collect();

        if !ansible_limit_nodes.is_empty() {
            // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

            (included, excluded) =
                crate::common::node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, ansible_limit_nodes);

            if !excluded.is_empty() {
                println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                            excluded,
                            hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                            hsm_groups_nodes);
                std::process::exit(-1);
            }
        } else {
            // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
            included = hsm_groups_nodes
        }
    } else {
        // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
        cfs_configuration_name = cli_apply_session
            .get_one::<String>("name")
            .unwrap()
            .to_string();
        included = ansible_limit_nodes
    }
    // * End Process/validate hsm group value (and ansible limit)

    // * Create CFS session
    let cfs_session_name = check_nodes_are_ready_to_run_cfs_session_and_run_cfs_session(
        &cfs_configuration_name,
        cli_apply_session
            .get_many("repo-path")
            .unwrap()
            .cloned()
            .collect(),
        // vec![cli_apply_session
        //     .get_one::<String>("repo-path")
        //     .unwrap()
        //     .to_string()],
        gitea_token,
        gitea_base_url,
        &shasta_token,
        &shasta_base_url,
        included.into_iter().collect::<Vec<String>>().join(","), // Convert Hashset to String with comma separator, need to convert to Vec first following https://stackoverflow.com/a/47582249/1918003
        cli_apply_session
            .get_one::<String>("ansible-verbosity")
            .unwrap()
            .parse()
            .unwrap(),
    )
    .await;

    let watch_logs = cli_apply_session.get_one::<bool>("watch-logs");

    if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");
        crate::cli::commands::log::session_logs(
            vault_base_url,
            cfs_session_name.unwrap().as_str(),
            None,
        )
        .await.unwrap();
    }
    // * End Create CFS session
}

pub async fn check_nodes_are_ready_to_run_cfs_session_and_run_cfs_session(
    config_name: &str,
    repos: Vec<String>,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    limit: String,
    ansible_verbosity: u8,
) -> Result<String, Box<dyn std::error::Error>> {
    
    // Get ALL sessions
    let cfs_sessions = http_client::get(
        &shasta_token,
        &shasta_base_url,
        None,
        None,
        None,
        None
    )
    .await?;

    let nodes_in_running_or_pending_cfs_session: Vec<&str> = cfs_sessions
        .iter()
        .filter(|cfs_session| cfs_session["status"]["session"]["status"].eq("pending") || cfs_session["status"]["session"]["status"].eq("running"))
        .flat_map(|cfs_session| {
            cfs_session["ansible"]["limit"]
                .as_str()
                .map(|limit| limit.split(','))
        })
        .flatten()
        .collect(); // TODO: remove duplicates

    log::info!("Nodes with cfs session running or pending: {:?}", nodes_in_running_or_pending_cfs_session);

    // NOTE: nodes can be a list of xnames or hsm group name

    // Convert limit (String with list of target nodes for new CFS session) into list of String
    let nodes_list: Vec<&str> = limit.split(',').map(|node| node.trim()).collect();

    // Check each node if it has a CFS session already running
    for node in nodes_list {
        if nodes_in_running_or_pending_cfs_session.contains(&node) {
            eprintln!(
                "The node {} from the list provided is already assigned to a running/pending CFS session. Please try again latter. Exitting", node
            );
            std::process::exit(-1);
        }
    }

    // Check nodes are ready to run a CFS layer
    let xnames: Vec<String> = limit
        .split(',')
        .map(|xname| String::from(xname.trim()))
        .collect();

    for xname in xnames {
        log::info!("Checking status of component {}", xname);

        let component_status =
            shasta_cfs_component::http_client::get(&shasta_token, &shasta_base_url, &xname).await?;
        let hsm_configuration_state =
            &hsm::http_client::get_component_status(&shasta_token, &shasta_base_url, &xname)
                .await?["State"];
        log::info!(
            "HSM component state for component {}: {}",
            xname,
            hsm_configuration_state.as_str().unwrap()
        );
        log::info!(
            "Is component enabled for batched CFS: {}",
            component_status["enabled"]
        );
        log::info!("Error count: {}", component_status["errorCount"]);

        if hsm_configuration_state.eq("On") || hsm_configuration_state.eq("Standby") {
            log::info!("There is an CFS session scheduled to run on this node. Pleas try again later. Aborting");
            std::process::exit(0);
        }
    }

    // Check local repos
    let mut layers_summary = vec![];

    for i in 0..repos.len() {
        // log::debug!("Local repo: {} state: {:?}", repo.path().display(), repo.state());
        // TODO: check each folder has a real git repo
        // TODO: check each folder has expected file name manta/shasta expects to find the main ansible playbook
        // TODO: a repo could param value could be a repo name, a filesystem path pointing to a repo or an url pointing to a git repo???
        // TODO: format logging on screen so it is more readable

        // Get repo from path
        let repo = match local_git_repo::get_repo(repos.get(i).unwrap()) {
            Ok(repo) => repo,
            Err(_) => {
                log::error!("Could not find a git repo in {}", repos[i]);
                std::process::exit(1);
            }
        };

        // Get last (most recent) commit
        let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

        log::info!("Checking local repo status ({})", &repo.path().display());

        // Check if all changes in local repo has been commited
        if !local_git_repo::untracked_changed_local_files(&repo).unwrap() {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Your local repo has changes not commited. Do you want to continue?")
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

        // Check site.yml file exists inside repo folder
        if !Path::new(repo.path()).exists() {
            log::error!(
                "site.yaml file does not exists in {}",
                repo.path().display()
            );
            std::process::exit(1);
        }

        // Get repo name
        let repo_ref_origin = repo.find_remote("origin").unwrap();

        log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());
        
        let repo_ref_origin_url = repo_ref_origin.url().unwrap();
        
        let repo_name = repo_ref_origin_url.substring(
            repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
            repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
        );

        let timestamp = local_last_commit.time().seconds();
        let tm = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
        log::debug!("\n\nCommit details to apply to CFS layer:\nCommit  {}\nAuthor: {}\nDate:   {}\n\n    {}\n", local_last_commit.id(), local_last_commit.author(), tm, local_last_commit.message().unwrap_or("no commit message"));

        let layer_summary = vec![i.to_string(), repo_name.to_string(), local_git_repo::untracked_changed_local_files(&repo)
        .unwrap()
        .to_string()];

        // layer_summary.push(i.to_string());
        // layer_summary.push(repo_name.to_string());
        // layer_summary.push(
        //     local_git_repo::untracked_changed_local_files(&repo)
        //         .unwrap()
        //         .to_string(),
        // );

        layers_summary.push(layer_summary);
    }

    log::debug!("Replacing '_' with '-' in repo name.");
    let cfs_configuration_name_formatted = format!("{}", str::replace(config_name, "_", "-"));

    println!("A CFS session {} is scheduled to run.", cfs_configuration_name_formatted);

    // Print CFS session/configuration layers summary on screen
    println!("Please review the following CFS layers:", );
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
        println!("Continue. Creating new CFS configuration and layer");
    } else {
        println!("Cancelled by user. Aborting.");
        std::process::exit(0);
    }

    // Check conflicts
    // git2_rs_utils::local::fetch_and_check_conflicts(&repo)?;
    // log::debug!("No conflicts");

    // Create CFS configuration
    let mut cfs_configuration = configuration::CfsConfiguration::new();

    for i in 0..repos.len() {
        // Get repo from path
        let repo = match local_git_repo::get_repo(repos.get(i).unwrap()) {
            Ok(repo) => repo,
            Err(_) => {
                log::error!("Could not find a git repo in {}", repos[i]);
                std::process::exit(1);
            }
        };

        // Get last (most recent) commit
        let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

        // Get repo name
        let repo_ref_origin = repo.find_remote("origin").unwrap();
        
        log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());
        
        let repo_ref_origin_url = repo_ref_origin.url().unwrap();
        
        let repo_name = repo_ref_origin_url.substring(
            repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
            repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
        );

        let mut api_url = "cray/".to_string();
        api_url.push_str(repo_name);

        // Check if repo and local commit id exists in Shasta cvs
        let shasta_commitid_details_resp = gitea::http_client::get_commit_details(
            &api_url,
            // &format!("/cray/{}", repo_name),
            &local_last_commit.id().to_string(),
            &gitea_token,
        )
        .await;

        // Check sync status between user face and shasta VCS
        let shasta_commitid_details: Value = match shasta_commitid_details_resp {
            Ok(_) => {
                log::debug!(
                    "Local latest commit id {} for repo {} exists in shasta",
                    local_last_commit.id(),
                    repo_name
                );
                shasta_commitid_details_resp.unwrap()
            }
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        };

        let mut clone_url = gitea_base_url.clone().to_string();
        clone_url.push_str("/cray/");
        clone_url.push_str(repo_name);

        // Create CFS layer
        let cfs_layer = configuration::Layer::new(
            // format!(
            //     // git repo url in shasta faced VCS
            //     "{}/cray/{}",
            //     gitea_base_url, // TODO: refactor this and move it to gitea mod
            //     repo_name
            // ),
            clone_url,
            String::from(shasta_commitid_details["sha"].as_str().unwrap()),
            format!(
                "{}-{}",
                repo_name.substring(0, repo_name.len()),
                chrono::offset::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ),
            String::from("site.yml"),
        );

        cfs_configuration = configuration::add_layer(cfs_layer, cfs_configuration);
    }

    log::info!("CFS configuration:\n{:#?}", cfs_configuration);

    // Update/PUT CFS configuration
    log::debug!("Create configuration and session name.");
    let cfs_configuration_resp = configuration::http_client::put(
        &shasta_token,
        &shasta_base_url,
        cfs_configuration,
        &cfs_configuration_name_formatted,
    )
    .await;

    let cfs_configuration_name = match cfs_configuration_resp {
        Ok(_) => {
            cfs_configuration_resp.as_ref().unwrap()["name"]
                .as_str()
                .unwrap()
        }
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    println!("CFS configuration name: {}", cfs_configuration_name);
    log::debug!("CFS configuration response: {:#?}", cfs_configuration_resp);

    // Create CFS session
    let cfs_session_name = format!(
        "{}-{}",
        cfs_configuration_name_formatted,
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    let session = shasta_cfs_session::CfsSession::new(
        cfs_session_name,
        cfs_configuration_name_formatted,
        Some(limit),
        ansible_verbosity,
    );

    log::debug!("Session:\n{:#?}", session);
    let cfs_session_resp =
        shasta_cfs_session::http_client::post(&shasta_token, &shasta_base_url, session).await;

        let cfs_session_name = match cfs_session_resp {
        Ok(_) => {
            cfs_session_resp.as_ref().unwrap()["name"].as_str().unwrap()
        }
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    println!("CFS session name: {}", cfs_session_name);
    log::debug!("CFS session response: {:#?}", cfs_session_resp);

    Ok(String::from(cfs_session_name))

    // Get pod name running the CFS session

    // Get list of ansible containers in pod running CFS session

    // Connect logs ????
}
