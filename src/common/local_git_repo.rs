// Code below inspired on https://github.com/rust-lang/git2-rs/issues/561
use std::path::{Path, PathBuf};

use anyhow::Context;
use git2::{Commit, ObjectType, Repository};

/// Open a local Git repository at the given path.
pub fn get_repo(repo_path: &str) -> Result<Repository, git2::Error> {
  let repo_root = PathBuf::from(repo_path);

  log::debug!("Checking repo on {}", repo_root.display());

  Repository::open(repo_root.as_os_str())
}

/// Get the most recent commit on the current branch.
pub fn get_last_commit(repo: &Repository) -> Result<Commit<'_>, git2::Error> {
  let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
  obj.into_commit().map_err(|obj| {
    git2::Error::from_str(&format!("Expected commit but got {:?}", obj.kind()))
  })
}

/// Return `true` if all tracked files are clean; `false`
/// if there are untracked or modified files.
pub fn untracked_changed_local_files(
  repo: &Repository,
) -> Result<bool, Box<dyn std::error::Error>> {
  let mut index = repo.index()?;

  log::debug!("Checking git index...");

  match index.add_all(
        ["."],
        git2::IndexAddOption::DEFAULT,
        Some(&mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = match repo.status_file(path) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::WT_NEW) {
                log::debug!(
                    "File not included in git index. Aborting process.\
                            Please run 'git status' to get list of file to work on"
                );
                -1
            } else {
                0
            }
        }),
    ) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Extract the repository name from the "origin" remote URL.
///
/// Finds the `origin` remote, reads its URL, takes the last
/// path segment, and strips any trailing `.git` suffix.
pub fn parse_repo_name_from_remote(
  repo: &Repository,
) -> Result<String, anyhow::Error> {
  let remote = repo
    .find_remote("origin")
    .context("Failed to find remote 'origin'")?;
  let url = remote
    .url()
    .context("Remote 'origin' URL is not valid UTF-8")?;
  let slash_pos = url.rfind('/').context("Remote URL has no '/' separator")?;
  Ok(url[slash_pos + 1..].trim_end_matches(".git").to_string())
}
