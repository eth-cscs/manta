//! Implements the `manta apply session` command.

use std::path::PathBuf;

use anyhow::{Context, Error, bail};
use clap::ArgMatches;

use crate::common::clap_ext::ArgMatchesExt;
use crate::common::confirm;
use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;

mod local_git_repo;

/// Gitea repository name prefix used by CFS.
const GITEA_REPO_NAME_PREFIX: &str = "cray/";

/// Create and run a CFS session on target nodes.
///
/// Authorization (target HSM group access + every xname in
/// `--ansible-limit` belonging to an accessible group) is enforced
/// server-side by `POST /api/v1/sessions`.
pub async fn exec(
  cli_apply_session: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let repo_path_vec: Vec<PathBuf> = cli_apply_session
    .get_many("repo-path")
    .context("'repo-path' argument not provided")?
    .cloned()
    .collect();

  let hsm_group_name_arg_opt = cli_apply_session.opt_str("group");

  let cfs_conf_sess_name_opt = cli_apply_session.opt_str("name");
  let playbook_file_name_opt = cli_apply_session.opt_str("playbook-name");

  let hsm_group_members_opt = cli_apply_session.opt_str("ansible-limit");
  let ansible_verbosity = cli_apply_session.opt_str("ansible-verbosity");

  let ansible_passthrough = cli_apply_session.opt_str("ansible-passthrough");

  let watch_logs: bool = cli_apply_session.get_flag("watch-logs");
  let timestamps: bool = cli_apply_session.get_flag("timestamps");
  let output_opt = cli_apply_session.opt_str("output");

  let _ = apply_session(
    ctx,
    token,
    SessionParams {
      session_name: cfs_conf_sess_name_opt,
      playbook: playbook_file_name_opt,
      hsm_group: hsm_group_name_arg_opt,
      repos: &repo_path_vec,
      ansible_limit: hsm_group_members_opt,
      ansible_verbosity,
      ansible_passthrough,
      watch_logs,
      timestamps,
      output: output_opt,
    },
  )
  .await?;

  Ok(())
}

struct SessionParams<'a> {
  session_name: Option<&'a str>,
  playbook: Option<&'a str>,
  hsm_group: Option<&'a str>,
  repos: &'a [PathBuf],
  ansible_limit: Option<&'a str>,
  ansible_verbosity: Option<&'a str>,
  ansible_passthrough: Option<&'a str>,
  watch_logs: bool,
  timestamps: bool,
  output: Option<&'a str>,
}

/// Creates a CFS session target dynamic.
///
/// Returns `(cfs_configuration_name, cfs_session_name)`.
async fn apply_session(
  ctx: &AppContext<'_>,
  shasta_token: &str,
  p: SessionParams<'_>,
) -> Result<(String, String), Error> {
  let cfs_conf_sess_name = p.session_name;
  let playbook_yaml_file_name_opt = p.playbook;
  let hsm_group_opt = p.hsm_group;
  let repos_paths = p.repos;
  let ansible_limit_opt = p.ansible_limit;
  let ansible_verbosity = p.ansible_verbosity;
  let ansible_passthrough = p.ansible_passthrough;
  let watch_logs = p.watch_logs;
  let timestamps = p.timestamps;
  let output_opt = p.output;
  let server_url = ctx.manta_server_url;

  // Check local repos (user interaction: confirm dialogs)
  let (repo_name_vec, repo_last_commit_id_vec) =
    check_local_repos(repos_paths)?;

  // Create CFS session via server
  let repo_names: Vec<&str> =
    repo_name_vec.iter().map(String::as_str).collect();
  let repo_commits: Vec<&str> =
    repo_last_commit_id_vec.iter().map(String::as_str).collect();
  let (cfs_configuration_name, cfs_session_name) =
    MantaClient::new(server_url, ctx.site_name)?
      .create_session(
        shasta_token,
        &crate::http_client::CreateSessionRequest {
          cfs_conf_sess_name,
          playbook_yaml_file_name: playbook_yaml_file_name_opt,
          hsm_group: hsm_group_opt,
          repo_names: &repo_names,
          repo_last_commit_ids: &repo_commits,
          ansible_limit: ansible_limit_opt,
          ansible_verbosity,
          ansible_passthrough,
        },
      )
      .await?;

  // Watch logs (CLI concern: println)
  if watch_logs {
    tracing::info!("Fetching logs ...");

    use tokio::io::AsyncBufReadExt as _;
    let client = MantaClient::new(server_url, ctx.site_name)?;
    let reader = client
      .stream_session_logs(shasta_token, &cfs_session_name, timestamps)
      .await
      .context("Failed to get CFS session log stream from server")?;
    let mut lines = reader.lines();
    while let Some(raw) = lines
      .next_line()
      .await
      .context("Failed to read CFS session log stream")?
    {
      if let Some(content) = raw.strip_prefix("data: ") {
        println!("{content}");
      }
    }
  }

  // Final result. In `--watch-logs` mode the streamed log lines above
  // have already gone to stdout; this summary appears after them. In
  // `--output json` mode the user should avoid `--watch-logs` to keep
  // the JSON on stdout uninterleaved with raw log content.
  action_result::print_with_data(
    &format!(
      "CFS session '{cfs_session_name}' created (configuration: '{cfs_configuration_name}')"
    ),
    &serde_json::json!({
      "cfs_configuration_name": cfs_configuration_name,
      "cfs_session_name": cfs_session_name,
    }),
    output_opt,
  )?;

  Ok((cfs_configuration_name, cfs_session_name))
}

fn check_local_repos(
  repos: &[PathBuf],
) -> Result<(Vec<String>, Vec<String>), Error> {
  let mut layers_summary = vec![];
  let mut repo_name_vec = Vec::new();
  let mut repo_last_commit_id_vec = Vec::new();

  for (i, repo_path) in repos.iter().enumerate() {
    let repo = match local_git_repo::get_repo(&repo_path.to_string_lossy()) {
      Ok(repo) => repo,
      Err(_) => {
        bail!(
          "Could not find a git repo in {}",
          repo_path.to_string_lossy()
        );
      }
    };

    let local_last_commit = local_git_repo::get_last_commit(&repo)
      .with_context(|| {
        format!(
          "Could not get last commit from repo at {}",
          repo_path.display()
        )
      })?;

    tracing::info!("Checking local repo status ({})", &repo.path().display());

    let all_committed = local_git_repo::untracked_changed_local_files(&repo)
      .map_err(|e| Error::msg(e.to_string()))
      .with_context(|| {
        format!(
          "Could not check local repo status at {}",
          repo_path.display()
        )
      })?;

    if !all_committed {
      if confirm::confirm(
        "Your local repo has uncommitted changes. Do you want to continue?",
        false,
      ) {
        println!(
          "Continue. Checking commit id {} against remote",
          local_last_commit.id()
        );
      } else {
        bail!("Operation cancelled by user");
      }
    }

    let repo_name_raw = local_git_repo::parse_repo_name_from_remote(&repo)
      .with_context(|| {
        format!(
          "Could not extract repo name from remote in {}",
          repo_path.display()
        )
      })?;

    let timestamp = local_last_commit.time().seconds();
    let tm = chrono::DateTime::from_timestamp(timestamp, 0)
      .context("Could not parse commit timestamp")?;

    tracing::debug!(
      "\n\nCommit details to apply to CFS layer:\nCommit  {}\nAuthor: {}\nDate:   {}\n\n    {}\n",
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

    repo_last_commit_id_vec.push(local_last_commit.id().to_string());

    let repo_name = GITEA_REPO_NAME_PREFIX.to_owned() + &repo_name_raw;
    repo_name_vec.push(repo_name);
  }

  // Print CFS session/configuration layers summary
  println!("Please review the following CFS layers:");
  for layer_summary in layers_summary {
    let layer_num = layer_summary
      .first()
      .map_or("?", std::string::String::as_str);
    let repo_name = layer_summary
      .get(1)
      .map_or("unknown", std::string::String::as_str);
    let committed = layer_summary
      .get(2)
      .map_or("unknown", std::string::String::as_str);
    println!(
      " - Layer-{layer_num}; repo name: {repo_name}; local changes committed: {committed}"
    );
  }

  if confirm::confirm(
    "Please review the layers and its order and confirm if proceed. Do you want to continue?",
    false,
  ) {
    println!("Continue. Creating new CFS configuration and layer(s)");
  } else {
    bail!("Operation cancelled by user");
  }

  Ok((repo_name_vec, repo_last_commit_id_vec))
}
