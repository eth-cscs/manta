use chrono::DateTime;
use substring::Substring;

use crate::common::local_git_repo;

pub async fn exec(
  shasta_root_cert: &[u8],
  gitea_base_url: &str,
  gitea_token: &str,
  repo_path: &str,
) {
  let mut exit_code = 0;

  println!("Validate local repo {}", repo_path);

  let repo = match local_git_repo::get_repo(&repo_path) {
    Ok(repo) => repo,
    Err(_) => {
      eprintln!("Could not find a git repo in {}", repo_path);
      std::process::exit(1);
    }
  };

  log::info!("Repo '{}' found", repo_path);

  // Get last (most recent) commit
  let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

  log::info!("Checking local repo status ({})", &repo.path().display());

  // Get repo name
  let repo_ref_origin = repo.find_remote("origin").unwrap();

  let repo_ref_origin_url = repo_ref_origin.url().unwrap();

  // TODO: do we still need the 'substring' crate? try to get rid of it
  let repo_name = repo_ref_origin_url
    .substring(
      repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
      repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
    )
    .trim_end_matches(".git");

  println!("Repository name: {}", repo_name);

  let head_commit_id = local_last_commit.id();
  let head_commit_summary = local_last_commit.summary();

  // println!("HEAD commit id: {}", head_commit_id);
  println!(
    "HEAD commit summary: {}",
    head_commit_summary.unwrap_or_default()
  );

  println!(
    "HEAD commit time: {}",
    DateTime::from_timestamp(local_last_commit.time().seconds(), 0).unwrap()
  );

  // Check if all changes in local repo has been commited locally
  if !local_git_repo::untracked_changed_local_files(&repo).unwrap() {
    println!("Local changes committed: ❌");
    exit_code = 1;
  } else {
    println!("Local changes committed: ✅");
  }

  let remote_ref_value_vec = csm_rs::common::gitea::http_client::get_all_refs(
    gitea_base_url,
    gitea_token,
    repo_name,
    shasta_root_cert,
  )
  .await
  .unwrap_or_default();

  // Get remote refs
  let remote_ref_vec: Vec<&str> = remote_ref_value_vec
    .iter()
    .map(|ref_value| ref_value["ref"].as_str().unwrap())
    .collect();

  // Validate HEAD local branch
  let branches = repo
    .branches(Some(git2::BranchType::Local))
    .expect("Something wrong while looking for local branches");

  for branch_rslt in branches.into_iter() {
    let (branch, _) = branch_rslt.unwrap();

    if branch.is_head() {
      let branch_name = branch.name().unwrap().unwrap();
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
      gitea_token,
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
  let local_tags = repo.tag_names(None).unwrap();

  for local_tag_rslt in &local_tags {
    let tag = local_tag_rslt.unwrap();
    if remote_ref_vec
      .iter()
      .any(|remote_tag| remote_tag.contains(&tag))
    {
      println!("tag {}: ✅", tag);
    } else {
      exit_code = 1;
      println!("tag {}: ❌", tag);
    }
  }

  println!("Repo synced? {}", exit_code == 0);

  if exit_code != 0 {
    std::process::exit(exit_code);
  }
}
