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

/// Extract the repository name from a remote URL string.
///
/// Takes the last path segment and strips any trailing `.git` suffix.
pub fn parse_repo_name_from_url(url: &str) -> Option<String> {
  let slash_pos = url.rfind('/')?;
  Some(url[slash_pos + 1..].trim_end_matches(".git").to_string())
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
  parse_repo_name_from_url(url).context("Remote URL has no '/' separator")
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── parse_repo_name_from_url ──

  #[test]
  fn https_url_with_git_suffix() {
    assert_eq!(
      parse_repo_name_from_url("https://github.com/org/my-repo.git"),
      Some("my-repo".to_string())
    );
  }

  #[test]
  fn https_url_without_git_suffix() {
    assert_eq!(
      parse_repo_name_from_url("https://github.com/org/my-repo"),
      Some("my-repo".to_string())
    );
  }

  #[test]
  fn ssh_url() {
    assert_eq!(
      parse_repo_name_from_url("git@github.com:org/my-repo.git"),
      Some("my-repo".to_string())
    );
  }

  #[test]
  fn url_with_trailing_slash_returns_empty() {
    assert_eq!(
      parse_repo_name_from_url("https://github.com/org/"),
      Some("".to_string())
    );
  }

  #[test]
  fn url_no_slash_returns_none() {
    assert_eq!(parse_repo_name_from_url("no-slash"), None);
  }

  #[test]
  fn deeply_nested_path() {
    assert_eq!(
      parse_repo_name_from_url("https://example.com/a/b/c/repo-name.git"),
      Some("repo-name".to_string())
    );
  }

  // ── git repo operations with temp dirs ──

  #[test]
  fn get_repo_valid_path() {
    let tmp = tempfile::tempdir().unwrap();
    Repository::init(tmp.path()).unwrap();
    let repo = get_repo(tmp.path().to_str().unwrap());
    assert!(repo.is_ok());
  }

  #[test]
  fn get_repo_invalid_path() {
    let result = get_repo("/nonexistent/path/to/repo");
    assert!(result.is_err());
  }

  #[test]
  fn get_last_commit_on_empty_repo_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Empty repo has no HEAD commit
    assert!(get_last_commit(&repo).is_err());
  }

  #[test]
  fn get_last_commit_after_initial_commit() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create an initial commit
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();
    let tree_id = {
      let mut index = repo.index().unwrap();
      index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo
      .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
      .unwrap();

    let commit = get_last_commit(&repo).unwrap();
    assert_eq!(commit.message(), Some("initial commit"));
  }

  #[test]
  fn parse_repo_name_from_remote_with_origin() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    repo
      .remote("origin", "https://github.com/org/test-repo.git")
      .unwrap();

    let name = parse_repo_name_from_remote(&repo).unwrap();
    assert_eq!(name, "test-repo");
  }

  #[test]
  fn parse_repo_name_from_remote_no_origin_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    assert!(parse_repo_name_from_remote(&repo).is_err());
  }

  #[test]
  fn untracked_files_clean_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create initial commit so repo has a valid HEAD
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo
      .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
      .unwrap();

    let clean = untracked_changed_local_files(&repo).unwrap();
    assert!(clean, "Clean repo should report true");
  }

  #[test]
  fn untracked_files_dirty_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create initial commit
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo
      .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
      .unwrap();

    // Create an untracked file
    std::fs::write(tmp.path().join("new_file.txt"), "content").unwrap();

    let clean = untracked_changed_local_files(&repo).unwrap();
    assert!(!clean, "Repo with untracked file should report false");
  }
}
