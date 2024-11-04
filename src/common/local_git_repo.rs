use std::path::{Path, PathBuf};

use git2::{Commit, ObjectType, Repository};

pub fn get_repo(repo_path: &str) -> Result<Repository, git2::Error> {
    let repo_root = PathBuf::from(repo_path);

    log::debug!("Checking repo on {}", repo_root.display());

    Repository::open(repo_root.as_os_str())
}

pub fn get_last_commit(repo: &Repository) -> Result<Commit<'_>, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

pub fn untracked_changed_local_files(
    repo: &Repository,
) -> Result<bool, Box<dyn std::error::Error>> {
    // use walkdir::WalkDir;

    // let root = std::env::current_dir().unwrap();

    // // Check tree/indexes Adding all files (git add)
    // for file in WalkDir::new(&root) {

    //     let file_aux = file.unwrap();

    //     if file_aux.metadata().unwrap().is_file() {

    //         // println!("{}", file_aux.path().display());
    //         // println!("{}", file_aux.path().strip_prefix(&root).unwrap().display());

    //         let status = repo.status_file(file_aux.path().strip_prefix(&root).unwrap()).unwrap();

    //         if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::WT_NEW) {
    //             println!("{}", file_aux.path().display());
    //             return Ok(false);
    //         }
    //     }
    // }

    // Ok(true)

    let mut index = repo.index().unwrap();

    log::debug!("Checking git index...");

    match index.add_all(
        ["."],
        git2::IndexAddOption::DEFAULT,
        Some(&mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = repo.status_file(path).unwrap();

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
