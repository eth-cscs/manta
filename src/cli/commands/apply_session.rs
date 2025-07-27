use std::path::PathBuf;

use futures::{AsyncBufReadExt, TryStreamExt};
use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_session::ApplySessionTrait, cfs::CfsTrait,
    hsm::component::ComponentTrait,
  },
  types::K8sDetails,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka, local_git_repo},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use dialoguer::{theme::ColorfulTheme, Confirm};
use substring::Substring;

/// Creates a CFS session target dynamic
/// Returns a tuple like (<cfs configuration name>, <cfs session name>)
pub async fn exec(
  backend: StaticBackendDispatcher,
  site: &str,
  gitea_token: &str,
  gitea_base_url: &str,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_conf_sess_name: Option<&String>,
  playbook_yaml_file_name_opt: Option<&String>,
  hsm_group_opt: Option<&String>,
  repos_paths: Vec<PathBuf>,
  ansible_limit_opt: Option<String>,
  ansible_verbosity: Option<String>,
  ansible_passthrough: Option<String>,
  watch_logs: bool,
  kafka_audit_opt: Option<&Kafka>,
  k8s: &K8sDetails,
) -> Result<(String, String), Error> {
  let ansible_limit = if let Some(ansible_limit) = ansible_limit_opt {
    // Convert user input to xname
    let node_metadata_available_vec = backend
      .get_node_metadata_available(shasta_token)
      .await
      .unwrap_or_else(|e| {
        eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
        std::process::exit(1);
      });

    let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
      &ansible_limit,
      false,
      node_metadata_available_vec,
    )
    .await
    .unwrap_or_else(|e| {
      eprintln!(
        "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
        e
      );
      std::process::exit(1);
    });

    Some(xname_vec.join(","))
  } else {
    None
  };

  // Check local repos
  let (repo_name_vec, repo_last_commit_id_vec) =
    check_local_repos(repos_paths.clone())?;

  let (cfs_configuration_name, cfs_session_name) = backend
    .apply_session(
      gitea_token,
      gitea_base_url,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      cfs_conf_sess_name,
      playbook_yaml_file_name_opt,
      hsm_group_opt,
      repo_name_vec,
      repo_last_commit_id_vec,
      ansible_limit.clone(),
      ansible_verbosity,
      ansible_passthrough,
    )
    .await?;

  // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
  // `manta logs`
  if watch_logs {
    log::info!("Fetching logs ...");

    let mut cfs_session_log_stream = backend
      .get_session_logs_stream(shasta_token, site, &cfs_session_name, k8s)
      .await?
      .lines();

    while let Some(line) = cfs_session_log_stream.try_next().await.unwrap() {
      println!("{}", line);
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": ansible_limit}, "group": vec![hsm_group_opt], "message": "Apply session"});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }

  Ok((cfs_configuration_name, cfs_session_name))
}

fn check_local_repos(
  repos: Vec<PathBuf>,
) -> Result<(Vec<String>, Vec<String>), Error> {
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
        .with_prompt(
          "Your local repo has uncommitted changes. Do you want to continue?",
        )
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

  let mut repo_name_vec = Vec::new();
  let mut repo_last_commit_id_vec = Vec::new();

  // Get layer names from local repos
  for repo_path in &repos {
    // Get repo from path
    let repo = match local_git_repo::get_repo(&repo_path.to_string_lossy()) {
      Ok(repo) => repo,
      Err(_) => {
        eprintln!(
          "Could not find a git repo in {}",
          repo_path.to_string_lossy()
        );
        std::process::exit(1);
      }
    };

    // Get last (most recent) commit
    let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

    repo_last_commit_id_vec.push(local_last_commit.id().to_string());

    // Get repo name
    let repo_ref_origin = repo.find_remote("origin").unwrap();

    log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());

    let repo_ref_origin_url = repo_ref_origin.url().unwrap();

    let repo_name = repo_ref_origin_url
      .substring(
        repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
        repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
      )
      .trim_end_matches(".git");

    let repo_name = "cray/".to_owned() + repo_name;

    repo_name_vec.push(repo_name);
  }

  Ok((repo_name_vec, repo_last_commit_id_vec))
}
