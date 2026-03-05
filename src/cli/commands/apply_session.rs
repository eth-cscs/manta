use std::path::PathBuf;

use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use futures::{AsyncBufReadExt, TryStreamExt};
use manta_backend_dispatcher::{
  interfaces::{apply_session::ApplySessionTrait, cfs::CfsTrait},
  types::K8sDetails,
};

use crate::common::{
  self,
  app_context::AppContext,
  audit,
  authentication::get_api_token,
  authorization::{get_groups_names_available, validate_target_hsm_members},
  jwt_ops, local_git_repo,
};

pub async fn exec(
  cli_apply_session: &ArgMatches,
  ctx: &AppContext<'_>,
  vault_base_url: &str,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;
  let configuration = ctx.configuration;

  let shasta_token = get_api_token(backend, site_name).await?;

  let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
    &shasta_token,
    vault_base_url,
    site_name,
  )
  .await?;

  let repo_path_vec: Vec<PathBuf> = cli_apply_session
    .get_many("repo-path")
    .context("'repo-path' argument not provided")?
    .cloned()
    .collect();

  let hsm_group_name_arg_opt: Option<&str> = cli_apply_session
    .get_one::<String>("hsm-group")
    .map(String::as_str);

  let cfs_conf_sess_name_opt: Option<&String> =
    cli_apply_session.get_one("name");
  let playbook_file_name_opt: Option<&String> =
    cli_apply_session.get_one("playbook-name");

  let hsm_group_members_opt: Option<&str> = cli_apply_session
    .get_one("ansible-limit")
    .map(String::as_str);
  let ansible_verbosity: Option<&String> =
    cli_apply_session.get_one("ansible-verbosity");

  let ansible_passthrough: Option<&String> =
    cli_apply_session.get_one("ansible-passthrough");

  let watch_logs: bool = cli_apply_session.get_flag("watch-logs");

  let timestamps: bool = cli_apply_session.get_flag("timestamps");

  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  if target_hsm_group_vec.is_empty() {
    bail!(
      "No HSM groups available for \
       this session",
    );
  }

  if let Some(ansible_limit) = hsm_group_members_opt {
    validate_target_hsm_members(
      backend,
      &shasta_token,
      &ansible_limit
        .split(',')
        .map(|xname| xname.trim().to_string())
        .collect::<Vec<String>>(),
    )
    .await?;
  }

  let site = configuration
    .sites
    .get(&configuration.site)
    .context(format!(
      "Site '{}' not found in configuration",
      &configuration.site
    ))?;

  let k8s_details = site
    .k8s
    .as_ref()
    .context("k8s section not found in configuration")?;

  let _ = apply_session(
    ctx,
    &gitea_token,
    &shasta_token,
    cfs_conf_sess_name_opt.map(String::as_str),
    playbook_file_name_opt.map(String::as_str),
    hsm_group_name_arg_opt,
    &repo_path_vec,
    hsm_group_members_opt,
    ansible_verbosity.map(String::as_str),
    ansible_passthrough.map(String::as_str),
    watch_logs,
    timestamps,
    k8s_details,
  )
  .await?;

  Ok(())
}

/// Creates a CFS session target dynamic
/// Returns a tuple like
/// (<cfs configuration name>, <cfs session name>)
#[allow(clippy::too_many_arguments)]
pub async fn apply_session(
  ctx: &AppContext<'_>,
  gitea_token: &str,
  shasta_token: &str,
  cfs_conf_sess_name: Option<&str>,
  playbook_yaml_file_name_opt: Option<&str>,
  hsm_group_opt: Option<&str>,
  repos_paths: &[PathBuf],
  ansible_limit_opt: Option<&str>,
  ansible_verbosity: Option<&str>,
  ansible_passthrough: Option<&str>,
  watch_logs: bool,
  timestamps: bool,
  k8s: &K8sDetails,
) -> Result<(String, String), Error> {
  let backend = ctx.backend.clone();
  let site = ctx.site_name;
  let gitea_base_url = ctx.gitea_base_url;
  let shasta_base_url = ctx.shasta_base_url;
  let shasta_root_cert = ctx.shasta_root_cert;
  let kafka_audit_opt = ctx.kafka_audit_opt;
  let ansible_limit = if let Some(ansible_limit) = ansible_limit_opt {
    // Convert user input to xname
    let xname_vec = common::node_ops::resolve_hosts_expression(
      &backend,
      shasta_token,
      ansible_limit,
      false,
    )
    .await?;

    Some(xname_vec.join(","))
  } else {
    None
  };

  // Check local repos
  let (repo_name_vec, repo_last_commit_id_vec) =
    check_local_repos(repos_paths)?;

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
      &repo_name_vec
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>(),
      &repo_last_commit_id_vec
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>(),
      ansible_limit.as_deref(),
      ansible_verbosity,
      ansible_passthrough,
    )
    .await?;

  // FIXME: refactor because this code is duplicated
  // in command `manta apply sat-file` and also in
  // `manta logs`
  if watch_logs {
    log::info!("Fetching logs ...");

    let mut cfs_session_log_stream = backend
      .get_session_logs_stream(
        shasta_token,
        site,
        &cfs_session_name,
        timestamps,
        k8s,
      )
      .await?
      .lines();

    while let Some(line) = cfs_session_log_stream.try_next().await.context(
      "Failed to read CFS session \
         log stream",
    )? {
      println!("{}", line);
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).context(
      "Could not extract name \
         from JWT token",
    )?;
    let user_id = jwt_ops::get_preferred_username(shasta_token).context(
      "Could not extract username \
           from JWT token",
    )?;

    let msg_json = serde_json::json!(
      {
        "user": {
          "id": user_id,
          "name": username
        },
        "host": {
          "hostname": ansible_limit
        },
        "group": vec![hsm_group_opt],
        "message": "Apply session"
      }
    );

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

  Ok((cfs_configuration_name, cfs_session_name))
}

fn check_local_repos(
  repos: &[PathBuf],
) -> Result<(Vec<String>, Vec<String>), Error> {
  let mut layers_summary = vec![];
  let mut repo_name_vec = Vec::new();
  let mut repo_last_commit_id_vec = Vec::new();

  for (i, repo_path) in repos.iter().enumerate() {
    // Get repo from path
    let repo = match local_git_repo::get_repo(&repo_path.to_string_lossy()) {
      Ok(repo) => repo,
      Err(_) => {
        bail!(
          "Could not find a git repo in {}",
          repo_path.to_string_lossy()
        );
      }
    };

    // Get last (most recent) commit
    let local_last_commit = local_git_repo::get_last_commit(&repo)
      .with_context(|| {
        format!(
          "Could not get last commit from \
             repo at {}",
          repo_path.display()
        )
      })?;

    log::info!("Checking local repo status ({})", &repo.path().display());

    // Check if all changes in local repo have been
    // committed locally
    let all_committed = local_git_repo::untracked_changed_local_files(&repo)
      .map_err(|e| Error::msg(e.to_string()))
      .with_context(|| {
        format!(
          "Could not check local repo \
               status at {}",
          repo_path.display()
        )
      })?;

    if !all_committed {
      if common::user_interaction::confirm(
        "Your local repo has uncommitted \
         changes. Do you want to continue?",
        false,
      ) {
        println!(
          "Continue. Checking commit id {} \
           against remote",
          local_last_commit.id()
        );
      } else {
        bail!("Cancelled by user. Aborting.");
      }
    }

    // Get repo name from 'origin' remote URL
    let repo_ref_origin = repo.find_remote("origin").with_context(|| {
      format!(
        "Could not find 'origin' remote in \
           repo at {}",
        repo_path.display()
      )
    })?;

    let repo_ref_origin_url = repo_ref_origin.url().with_context(|| {
      format!(
        "Remote 'origin' URL is not valid \
           UTF-8 in repo at {}",
        repo_path.display()
      )
    })?;

    log::info!("Repo ref origin URL: {}", repo_ref_origin_url);

    let slash_pos = repo_ref_origin_url.rfind('/').with_context(|| {
      format!(
        "Remote URL '{}' has unexpected \
           format (no '/' found)",
        repo_ref_origin_url
      )
    })?;
    let repo_name_raw = &repo_ref_origin_url[slash_pos + 1..];

    let timestamp = local_last_commit.time().seconds();
    let tm = chrono::DateTime::from_timestamp(timestamp, 0)
      .context("Could not parse commit timestamp")?;

    log::debug!(
      "\n\nCommit details to apply to CFS \
       layer:\nCommit  {}\nAuthor: {}\n\
       Date:   {}\n\n    {}\n",
      local_last_commit.id(),
      local_last_commit.author(),
      tm,
      local_last_commit.message().unwrap_or("no commit message")
    );

    layers_summary.push(vec![
      i.to_string(),
      repo_name_raw.to_string(),
      all_committed.to_string(),
    ]);

    // Collect data for CFS configuration creation
    repo_last_commit_id_vec.push(local_last_commit.id().to_string());

    let repo_name = "cray/".to_owned() + repo_name_raw.trim_end_matches(".git");
    repo_name_vec.push(repo_name);
  }

  // Print CFS session/configuration layers summary
  println!("Please review the following CFS layers:",);
  for layer_summary in layers_summary {
    let layer_num = layer_summary.first().map(|s| s.as_str()).unwrap_or("?");
    let repo_name = layer_summary
      .get(1)
      .map(|s| s.as_str())
      .unwrap_or("unknown");
    let committed = layer_summary
      .get(2)
      .map(|s| s.as_str())
      .unwrap_or("unknown");
    println!(
      " - Layer-{}; repo name: {}; \
       local changes committed: {}",
      layer_num, repo_name, committed
    );
  }

  if common::user_interaction::confirm(
    "Please review the layers and its order and \
     confirm if proceed. Do you want to continue?",
    false,
  ) {
    println!(
      "Continue. Creating new CFS configuration \
       and layer(s)"
    );
  } else {
    bail!("Cancelled by user. Aborting.");
  }

  Ok((repo_name_vec, repo_last_commit_id_vec))
}
