mod local {
    pub fn untracked_changed_local_files() {

        let files = vec![];

        // Check if conflicts
        // TODO: This may be the wrong place to check if there are conflicts (maybe too early) and we need to fetch data from remote
        if index.has_conflicts() {
            log::error!("THERE ARE CONFLICTS!!!!!");

            std::process::exit(1);
        }

        // Check tree/indexes Adding all files (git add)
        log::debug!("Running 'git add'");

        index.add_all(&["."], git2::IndexAddOption::DEFAULT, Some(&mut |path: &Path, _matched_spec: &[u8]| {

                    let status = repo.status_file(path).unwrap();

                    let ret = if status.contains(git2::Status::WT_MODIFIED) {
                        log::debug!(" - Modified file: '{}'", path.display());
                    } else if status.contains(git2::Status::WT_NEW) {
                        log::debug!(" - New file: '{}'", path.display());
                    };
                }),
            )
            .unwrap();

        
    }
}
