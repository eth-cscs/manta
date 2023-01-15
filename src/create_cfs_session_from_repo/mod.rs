use std::path::Path;

use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::shasta::cfs::configuration;
use crate::shasta::cfs::session::http_client;
use crate::shasta::hsm;
use crate::{shasta_cfs_component, shasta_cfs_session};
use k8s_openapi::chrono;
use serde_json::Value;
use substring::Substring;

use crate::{gitea, local_git_repo};

pub async fn run(
    config_name: &str,
    repos: Vec<String>,
    gitea_token: String,
    gitea_base_url: String,
    shasta_token: String,
    shasta_base_url: String,
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
    let nodes_list: Vec<&str> = limit.split(",").map(|node| node.trim()).collect();

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
        let repo = match local_git_repo::get_repo(repos[i].clone()) {
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

        let mut layer_summary = vec![];

        layer_summary.push(i.to_string());
        layer_summary.push(repo_name.to_string());
        layer_summary.push(
            local_git_repo::untracked_changed_local_files(&repo)
                .unwrap()
                .to_string(),
        );

        layers_summary.push(layer_summary);
    }

    // Print CFS session/configuration layers summary on screen
    println!("A new CFS session is going to be created with the following layers:");
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
        let repo = match local_git_repo::get_repo(repos[i].clone()) {
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

        // Check if repo and local commit id exists in Shasta cvs
        let shasta_commitid_details_resp = gitea::http_client::get_commit_details(
            &format!("/cray/{}", repo_name),
            &local_last_commit.id().to_string(),
            &gitea_token,
        )
        .await;

        // Check sync status between user face and shasta VCS
        let shasta_commitid_details: Value;
        match shasta_commitid_details_resp {
            Ok(_) => {
                log::debug!(
                    "Local latest commit id {} for repo {} exists in shasta",
                    local_last_commit.id(),
                    repo_name
                );
                shasta_commitid_details = shasta_commitid_details_resp.unwrap();
            }
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }

        // Create CFS layer
        let cfs_layer = configuration::Layer::new(
            format!(
                // git repo url in shasta faced VCS
                "{}/cray/{}",
                gitea_base_url, // TODO: refactor this and move it to gitea mod
                repo_name
            ),
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
    log::debug!("Replacing '_' with '-' in repo name and create configuration and session name.");
    let cfs_configuration_name_formatted = format!("m-{}", str::replace(config_name, "_", "-"));
    let cfs_configuration_resp = configuration::http_client::put(
        &shasta_token,
        &shasta_base_url,
        cfs_configuration,
        &cfs_configuration_name_formatted,
    )
    .await;

    let cfs_configuration_name;
    match cfs_configuration_resp {
        Ok(_) => {
            cfs_configuration_name = cfs_configuration_resp.as_ref().unwrap()["name"]
                .as_str()
                .unwrap();
        }
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    log::info!("CFS configuration name: {}", cfs_configuration_name);
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

    let cfs_session_name;
    match cfs_session_resp {
        Ok(_) => {
            cfs_session_name = cfs_session_resp.as_ref().unwrap()["name"].as_str().unwrap();
        }
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    log::info!("CFS session name: {}", cfs_session_name);
    log::debug!("CFS session response: {:#?}", cfs_session_resp);

    Ok(String::from(cfs_session_name))

    // Get pod name running the CFS session

    // Get list of ansible containers in pod running CFS session

    // Connect logs ????
}
