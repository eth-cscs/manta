use std::path::Path;

use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::Repository;
use k8s_openapi::chrono;
use serde_json::Value;
use substring::Substring;
use crate::{shasta_cfs_session, shasta_cfs_component, shasta_hsm};

use crate::{git2_rs_utils, shasta_vcs_utils, shasta_cfs_configuration};

pub async fn run(repo: Repository, gitea_token: String, shasta_token:String, shasta_base_url: String, limit: String, ansible_verbosity: u8) -> Result<String, Box<dyn std::error::Error>> {

    let component_status = shasta_cfs_component::http_client::get(&shasta_token, &shasta_base_url, &limit).await?;
    let hsm_configuration_state = &shasta_hsm::http_client::get_component_status(&shasta_token, &shasta_base_url, &limit).await?["State"];
    log::info!("HSM component state for component {}: {}", limit, hsm_configuration_state.as_str().unwrap());
    log::info!("Is component enabled for batched CFS: {}", component_status["enabled"]);
    log::info!("Error count: {}", component_status["errorCount"]);

    if hsm_configuration_state.eq("On") || hsm_configuration_state.eq("Standby") {
        log::info!("There is an CFS session scheduled to run on this node. Pleas try again later. Aborting");
        std::process::exit(0);
    }

    // Get last (most recent) commit
    let local_last_commit = git2_rs_utils::local::get_last_commit(&repo).unwrap();

    log::info!("Checking local repo status ({})", &repo.path().display());

    if !git2_rs_utils::local::untracked_changed_local_files(&repo).unwrap() {
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
        log::error!("site.yaml file does not exists in {}", repo.path().display());
        std::process::exit(1);
    }

    // Get repo name
    let repo_ref_origin = repo.find_remote("origin").unwrap();
    log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());
    let repo_ref_origin_url = repo_ref_origin.url().unwrap();
    let repo_name = repo_ref_origin_url.substring(
        repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
        repo_ref_origin_url.len()
        // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
    );

    log::debug!("Repo name: {}", repo_name);

    // Check if repo and local commit id exists in Shasta cvs
    let shasta_commitid_details_resp = shasta_vcs_utils::http_client::get_commit_details(
        &format!("cray/{}", repo_name),
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
        },
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
    
    let timestamp = local_last_commit.time().seconds();
    let tm = chrono::NaiveDateTime::from_timestamp(timestamp, 0);
    log::debug!("\nCommit details to apply to CFS layer:\nCommit  {}\nAuthor: {}\nDate:   {}\n\n    {}", local_last_commit.id(), local_last_commit.author(), tm, local_last_commit.message().unwrap_or("no commit message"));

    // Check conflicts
    // git2_rs_utils::local::fetch_and_check_conflicts(&repo)?;
    // log::debug!("No conflicts");

    // Create CFS layer
    let cfs_layer = shasta_cfs_configuration::Layer::new(
        String::from(format!(  // git repo url in shasta faced VCS
            "https://api-gw-service-nmn.local/vcs/cray/{}",
            repo_name
        )),
        // String::from(repo_ref_origin_url), // git repo url in user faced VCS
        String::from(shasta_commitid_details["sha"].as_str().unwrap()),
        String::from(format!(
            "{}-{}",
            repo_name.substring(1, repo_name.len()),
            chrono::offset::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        )),
        String::from("site.yml"),
    );

    // Create CFS configuration
    let mut cfs_configuration = shasta_cfs_configuration::Configuration::new();

    cfs_configuration = shasta_cfs_configuration::add_layer(cfs_layer, cfs_configuration);

    log::debug!("CFS configuration:\n{:#?}", cfs_configuration);

    // Update/PUT CFS configuration
    log::debug!("Replacing '_' with '-' in repo name and create configuration and session name.");
    let cfs_object_name = format!("m-{}", str::replace(repo_name, "_", "-"));
    let cfs_configuration_resp = shasta_cfs_configuration::http_client::put(
        &shasta_token,
        &shasta_base_url,
        cfs_configuration,
        &cfs_object_name,
    )
    .await;

    let cfs_configuration_name;
    match cfs_configuration_resp {
        Ok(_) => {
            cfs_configuration_name = cfs_configuration_resp.as_ref().unwrap()["name"].as_str().unwrap();
        },
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
        cfs_object_name,
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    let session = shasta_cfs_session::Session::new(
        cfs_session_name,
        cfs_object_name,
        Some(limit),
        ansible_verbosity
    );

    log::debug!("Session:\n{:#?}", session);
    let cfs_session_resp =
        shasta_cfs_session::http_client::post(&shasta_token, &shasta_base_url, session).await;

    let cfs_session_name;
    match cfs_session_resp {
        Ok(_) => {
            cfs_session_name = cfs_session_resp.as_ref().unwrap()["name"].as_str().unwrap();
        },
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
