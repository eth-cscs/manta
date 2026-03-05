use anyhow::{Context, Error, bail};
use chrono::DateTime;
use serde_json::Value;

use crate::common::{
  authentication::get_api_token, local_git_repo,
  vault::http_client::fetch_shasta_vcs_token,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Validate a local Git repo against the remote VCS.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_root_cert: &[u8],
  vault_base_url: Option<&str>,
  gitea_base_url: &str,
  repo_path: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  let gitea_token = fetch_shasta_vcs_token(
    &shasta_token,
    vault_base_url.context("vault base url is mandatory")?,
    site_name,
  )
  .await
  .context("Failed to fetch VCS token from vault")?;

  let mut exit_code = 0;

  println!("Validate local repo {}", repo_path);

  let repo = local_git_repo::get_repo(repo_path)
    .with_context(|| format!("Could not open git repo in {}", repo_path))?;

  log::info!("Repo '{}' found", repo_path);

  // Get last (most recent) commit
  let local_last_commit = local_git_repo::get_last_commit(&repo)
    .context("Failed to get last commit")?;

  log::info!("Checking local repo status ({})", &repo.path().display());

  // Get repo name
  let repo_ref_origin = repo
    .find_remote("origin")
    .context("Failed to find remote 'origin'")?;

  let repo_ref_origin_url = repo_ref_origin
    .url()
    .context("Remote 'origin' URL is not valid UTF-8")?;

  let slash_pos = repo_ref_origin_url
    .rfind('/')
    .context("Remote URL has no '/' separator")?;
  let repo_name = repo_ref_origin_url[slash_pos + 1..].trim_end_matches(".git");

  println!("Repository name: {}", repo_name);

  let head_commit_id = local_last_commit.id();
  let head_commit_summary = local_last_commit.summary();

  println!(
    "HEAD commit summary: {}",
    head_commit_summary.unwrap_or_default()
  );

  println!(
    "HEAD commit time: {}",
    DateTime::from_timestamp(local_last_commit.time().seconds(), 0,)
      .context("Failed to parse commit timestamp")?
  );

  // Check if all changes in local repo has been commited locally
  if !local_git_repo::untracked_changed_local_files(&repo)
    .map_err(|e| anyhow::anyhow!("{e}"))
    .context("Failed to check for untracked/changed files")?
  {
    println!("Local changes committed: ❌");
    exit_code = 1;
  } else {
    println!("Local changes committed: ✅");
  }

  let remote_ref_value_vec = csm_rs::common::gitea::http_client::get_all_refs(
    gitea_base_url,
    &gitea_token,
    repo_name,
    shasta_root_cert,
  )
  .await
  .unwrap_or_default();

  // Get remote refs
  let remote_ref_vec: Vec<&str> = remote_ref_value_vec
    .iter()
    .filter_map(|ref_value| ref_value.get("ref").and_then(Value::as_str))
    .collect();

  // Validate HEAD local branch
  let branches = repo
    .branches(Some(git2::BranchType::Local))
    .context("Failed to list local branches")?;

  for branch_rslt in branches.into_iter() {
    let (branch, _) = match branch_rslt {
      Ok(b) => b,
      Err(e) => {
        log::warn!("Failed to read branch: {}", e);
        continue;
      }
    };

    if branch.is_head() {
      let branch_name = match branch.name() {
        Ok(Some(name)) => name,
        _ => {
          log::warn!("Failed to read branch name");
          continue;
        }
      };
      if remote_ref_vec
        .iter()
        .any(|remote_ref| remote_ref.contains(branch_name))
      {
        println!("HEAD branch '{}': ✅", branch_name);
      } else {
        exit_code = 1;
        println!("HEAD branch '{}': ❌", branch_name);
      }
    }
  }

  // Validate most recent commit id in local repo agaisnt remote repo
  let gitea_commit_details =
    csm_rs::common::gitea::http_client::get_commit_details(
      gitea_base_url,
      repo_name,
      &head_commit_id.to_string(),
      &gitea_token,
      shasta_root_cert,
    )
    .await;

  if gitea_commit_details.is_ok() {
    println!("HEAD commit id {}: ✅", head_commit_id);
  } else {
    println!("HEAD commit id {}: ❌", head_commit_id);
    exit_code = 1;
  }

  // Validate tags
  let local_tags = repo.tag_names(None).context("Failed to list local tags")?;

  for local_tag_rslt in local_tags.iter() {
    let tag = match local_tag_rslt {
      Some(t) => t,
      None => {
        log::warn!("Failed to read tag name");
        continue;
      }
    };
    if remote_ref_vec
      .iter()
      .any(|remote_tag| remote_tag.contains(tag))
    {
      println!("tag {}: ✅", tag);
    } else {
      exit_code = 1;
      println!("tag {}: ❌", tag);
    }
  }

  println!("Repo synced? {}", exit_code == 0);

  if exit_code != 0 {
    bail!(
      "Local repository is not in sync with \
       remote repository",
    );
  }

  Ok(())
}
