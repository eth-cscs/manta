//! Implements the `manta run session` command.
//!
//! Build, validate, and run a one-off CFS session driven by local Git
//! repos:
//!
//! 1. For every `--repo-path`, open the git repo, capture name + HEAD
//!    commit id, and (interactively) confirm any uncommitted changes.
//!    Implemented in [`check_local_repos`], with git helpers in
//!    [`local_git_repo`].
//! 2. POST `/api/v1/sessions` (`create_session`) with the layer list,
//!    the optional `--playbook`, `--ansible-limit`, `--ansible-verbosity`,
//!    and `--ansible-passthrough`. Server returns the created CFS
//!    configuration and session names.
//! 3. If `--watch-logs` is set, stream the session log over SSE
//!    (`stream_session_logs`) to stdout, optionally with timestamps.
//!
//! `--dry-run` prints the would-be request via
//! [`crate::output::action_result::preview_request`] and skips both the
//! POST and the log stream; the repo walk still runs (so the preview
//! reflects the would-be commit ids) but uncommitted files are
//! tolerated with a warning instead of a prompt.

use std::path::PathBuf;

use anyhow::{Context, Error, bail};
use clap::ArgMatches;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::common::confirm;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::CreateSessionRequest;
use crate::output::action_result;

mod local_git_repo;

/// Gitea repository name prefix used by CFS.
const GITEA_REPO_NAME_PREFIX: &str = "cray/";

/// Create and run a CFS session on target nodes.
///
/// Authorization (target HSM group access + every xname in
/// `--ansible-limit` belonging to an accessible group) is enforced
/// server-side by `POST /api/v1/sessions`.
///
/// # Errors
///
/// Returns an error when `--repo-path` is missing, when [`run_session`]
/// returns an error (repo walk, prompt declined, HTTP failure, or log
/// stream failure).
pub async fn exec(
  cli_run_session: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let repo_path_vec: Vec<PathBuf> = cli_run_session
    .get_many("repo-path")
    .context("'repo-path' argument not provided")?
    .cloned()
    .collect();

  let group_name_arg_opt = cli_run_session.opt_str("group");

  let cfs_conf_sess_name_opt = cli_run_session.opt_str("name");
  let playbook_file_name_opt = cli_run_session.opt_str("playbook-name");

  let group_members_opt = cli_run_session.opt_str("ansible-limit");
  let ansible_verbosity = cli_run_session.opt_str("ansible-verbosity");

  let ansible_passthrough = cli_run_session.opt_str("ansible-passthrough");

  let watch_logs: bool = cli_run_session.get_flag("watch-logs");
  let timestamps: bool = cli_run_session.get_flag("timestamps");
  let output_opt = cli_run_session.opt_str("output");
  let dry_run: bool = cli_run_session.get_flag("dry-run");

  run_session(
    ctx,
    token,
    SessionParams {
      session_name: cfs_conf_sess_name_opt,
      playbook: playbook_file_name_opt,
      group_name: group_name_arg_opt,
      repos: &repo_path_vec,
      ansible_limit: group_members_opt,
      ansible_verbosity,
      ansible_passthrough,
      watch_logs,
      timestamps,
      output: output_opt,
      dry_run,
    },
  )
  .await
}

struct SessionParams<'a> {
  session_name: Option<&'a str>,
  playbook: Option<&'a str>,
  group_name: Option<&'a str>,
  repos: &'a [PathBuf],
  ansible_limit: Option<&'a str>,
  ansible_verbosity: Option<&'a str>,
  ansible_passthrough: Option<&'a str>,
  watch_logs: bool,
  timestamps: bool,
  output: Option<&'a str>,
  dry_run: bool,
}

/// Create a dynamic-target CFS session: pushes the local repos'
/// HEADs to gitea, POSTs `/sessions`, and (if `watch_logs`) streams
/// the session log to stdout via SSE before printing the action
/// result.
///
/// The created CFS configuration and session names are already
/// reported through `action_result::print_with_data`, so this
/// function returns `()` rather than handing the names back to a
/// caller that wouldn't have anything to do with them.
async fn run_session(
  ctx: &AppContext<'_>,
  shasta_token: &str,
  p: SessionParams<'_>,
) -> Result<(), Error> {
  let cfs_conf_sess_name = p.session_name;
  let playbook_yaml_file_name_opt = p.playbook;
  let group_name_opt = p.group_name;
  let repos_paths = p.repos;
  let ansible_limit_opt = p.ansible_limit;
  let ansible_verbosity = p.ansible_verbosity;
  let ansible_passthrough = p.ansible_passthrough;
  let watch_logs = p.watch_logs;
  let timestamps = p.timestamps;
  let output_opt = p.output;

  // Check local repos (user interaction: confirm dialogs)
  let (repo_name_vec, repo_last_commit_id_vec) =
    check_local_repos(repos_paths, p.dry_run)?;

  let req = CreateSessionRequest {
    cfs_conf_sess_name: cfs_conf_sess_name.map(str::to_string),
    playbook_yaml_file_name: playbook_yaml_file_name_opt.map(str::to_string),
    hsm_group: group_name_opt.map(str::to_string),
    repo_names: repo_name_vec,
    repo_last_commit_ids: repo_last_commit_id_vec,
    ansible_limit: ansible_limit_opt.map(str::to_string),
    ansible_verbosity: ansible_verbosity.map(str::to_string),
    ansible_passthrough: ansible_passthrough.map(str::to_string),
  };

  if p.dry_run {
    return action_result::preview_request(
      "POST",
      "/sessions",
      &req,
      output_opt,
    );
  }

  // Create CFS session via server
  let client = MantaClient::from_app_ctx(ctx, Some(shasta_token))?;
  let created = client
    .openapi
    .create_session(client.site_name(), &req)
    .await
    .into_anyhow()?;
  let cfs_configuration_name = created.configuration_name;
  let cfs_session_name = created.session_name;

  // Watch logs (CLI concern: println)
  if watch_logs {
    tracing::info!("Fetching logs ...");

    use tokio::io::AsyncBufReadExt as _;
    let reader = client
      .stream_session_logs(&cfs_session_name, timestamps)
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

  Ok(())
}

/// Walk every `--repo` path: open the git repo, capture its name and
/// HEAD commit, and check for uncommitted changes. Returns parallel
/// `(repo_names, last_commit_ids)` vectors used to build the
/// `CreateSessionRequest`.
///
/// `dry_run` controls only the interactive policy when a repo has
/// uncommitted files: in dry-run we log a `tracing::warn!` and
/// continue (so the preview reflects the would-be commit ids); in a
/// live run we surface a confirmation prompt because pushing a dirty
/// HEAD would publish work-in-progress as a CFS layer commit. The
/// flag is **not** a "skip the validation" switch — every repo still
/// gets opened, named, and inspected on both paths.
fn check_local_repos(
  repos: &[PathBuf],
  dry_run: bool,
) -> Result<(Vec<String>, Vec<String>), Error> {
  let mut layers_summary: Vec<String> = Vec::new();
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
      if dry_run {
        tracing::warn!(
          "[dry-run] Local repo at {} has uncommitted changes; \
           preview reflects the last committed id {}.",
          repo_path.display(),
          local_last_commit.id()
        );
      } else if confirm::confirm(
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

    layers_summary.push(format!(
      " - Layer-{i}; repo name: {repo_name_raw}; local changes committed: {all_committed}"
    ));

    repo_last_commit_id_vec.push(local_last_commit.id().to_string());

    let repo_name = GITEA_REPO_NAME_PREFIX.to_owned() + &repo_name_raw;
    repo_name_vec.push(repo_name);
  }

  // Print CFS session/configuration layers summary
  println!("Please review the following CFS layers:");
  for line in &layers_summary {
    println!("{line}");
  }

  if !dry_run {
    if !confirm::confirm(
      "Please review the layers and its order and confirm if proceed. Do you want to continue?",
      false,
    ) {
      bail!("Operation cancelled by user");
    }
    println!("Continue. Creating new CFS configuration and layer(s)");
  }

  Ok((repo_name_vec, repo_last_commit_id_vec))
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta run session` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "run",
      "session",
      "--name",
      "test-session",
      "--repo-path",
      "/tmp/repo",
      "--ansible-limit",
      "x1000c0s0b0n0",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `run session`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "run",
      "session",
      "--name",
      "test-session",
      "--repo-path",
      "/tmp/repo",
      "--ansible-limit",
      "x1000c0s0b0n0",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
