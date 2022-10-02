// Code below inspired on https://github.com/rust-lang/git2-rs/issues/561

pub mod local {

    use std::path::Path;
    use std::io::{self, Write};

    use git2::{Repository, PushOptions, Remote};

    pub fn get_repo() -> Repository {
                    
        // Get repo on current dir (pwd)
        let repo_root = std::env::current_dir().unwrap();
        log::debug!("Checking repo on {}", repo_root.display());
        
        let repo = Repository::open(repo_root.as_os_str()).expect("Couldn't open repository");

        repo
    }

    pub fn untracked_changed_local_files(repo: Repository) {
        
        let index = repo.index().unwrap();

        use walkdir::WalkDir;

        // Check if conflicts
        // TODO: This may be the wrong place to check if there are conflicts (maybe too early) and we need to fetch data from remote
        if index.has_conflicts() {
            log::error!("THERE ARE CONFLICTS!!!!!");

            std::process::exit(1);
        }

        // Check tree/indexes Adding all files (git add)
        log::debug!("Running 'git add'");

        for file in WalkDir::new(".").into_iter().filter_map(|file| file.ok()) {

            if file.metadata().unwrap().is_file() {

                println!("{}", file.path().display());

                let status = repo.status_file(file.path()).unwrap();

                if status.contains(git2::Status::WT_MODIFIED) {
                    log::debug!(" - Modified file: '{}'", file.path().display());
                } else if status.contains(git2::Status::WT_NEW) {
                    log::debug!(" - New/added file: '{}'", file.path().display());
                };
            }
        }        
    }

    pub fn add_all(repo: &Repository) {

        let mut index = repo.index().unwrap();

        log::debug!("Running 'git add'");

        index.add_all(&["."], git2::IndexAddOption::DEFAULT, Some(&mut |path: &Path, _matched_spec: &[u8]| -> i32 {

            let status = repo.status_file(path).unwrap();
    
            let ret = if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::WT_NEW)
            {
                log::debug!(" - Adding file: '{}' with status {:?}", path.display(), status);

                0
            } else {
                log::debug!(" - NOT adding file: '{}' with status {:?}", path.display(), status);

                1
            };

            ret
        }))
            .unwrap();
        index.write().unwrap();
    }

    pub fn commit(repo: &Repository) {
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let signature = repo.signature().unwrap();
        let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(
            Some("HEAD"), 
            &signature, 
            &signature, 
            "testing git2-rs... commit created programatically...", 
            &tree, 
            &[&parent_commit]).unwrap();
    }

    pub fn push(mut remote: Remote) -> Result<(), git2::Error> {
        // Configure callbacks for push operation
        let mut callbacks = git2::RemoteCallbacks::new();

        // TODO: CLEAN THIS!!!
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            log::debug!("url is: {}", _url);
            log::debug!("username from url is: {}", _username_from_url.unwrap_or("Not defined")); // IMPORTANT: username from url is None because .git/config has https address 'url = https://git.cscs.ch/msopena/manta.git' 
            log::debug!("allowed types are: {:#?}", _allowed_types);
                        
            git2::Cred::userpass_plaintext("msopena", "MasberLugano0720") // IMPORTANT: this with combination of .git/config having an https address 'url = https://git.cscs.ch/msopena/manta.git' makes library to switch to CredentialType::USER_PASS_PLAINTEXT
        });
        
        callbacks.push_update_reference(|_reference_name, callback_status| {
            log::debug!("reference name: {}", _reference_name);
            log::debug!("callback status: {}", callback_status.unwrap_or("Not defined"));
        
            Ok(())
        });
        
        // Configure push options
        let po = &mut PushOptions::default();
        po.remote_callbacks(callbacks);
        
        // Push
        remote.push(&["+refs/heads/main","+refs/heads/apply-dynamic-target-session"], Some(po))

    }

    pub fn fetch<'a>(repo: &'a git2::Repository, refs: &[&str], remote: &'a mut git2::Remote,) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {

        let mut cb = git2::RemoteCallbacks::new();
    
        // TODO: CLEAN THIS!!!
        cb.credentials(|_url, _username_from_url, _allowed_types| {
            log::debug!("url is: {}", _url);
            log::debug!("username from url is: {}", _username_from_url.unwrap_or("Not defined")); // IMPORTANT: username from url is None because .git/config has https address 'url = https://git.cscs.ch/msopena/manta.git' 
            log::debug!("allowed types are: {:#?}", _allowed_types);
                        
            git2::Cred::userpass_plaintext("msopena", "MasberLugano0720") // IMPORTANT: this with combination of .git/config having an https address 'url = https://git.cscs.ch/msopena/manta.git' makes library to switch to CredentialType::USER_PASS_PLAINTEXT
        });

        // Print out our transfer progress.
        cb.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                print!(
                    "Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                print!(
                    "Received {}/{} objects ({}) in {} bytes\r",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.indexed_objects(),
                    stats.received_bytes()
                );
            }
            io::stdout().flush().unwrap();
            true
        });
    
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(cb);
        // Always fetch all tags.
        // Perform a download and also update tips
        fo.download_tags(git2::AutotagOption::All);
        println!("Fetching {} for repo", remote.name().unwrap());
        remote.fetch(refs, Some(&mut fo), None)?;
    
        // If there are local objects (we got a thin pack), then tell the user
        // how many objects we saved from having to cross the network.
        let stats = remote.stats();
        if stats.local_objects() > 0 {
            println!(
                "\rReceived {}/{} objects in {} bytes (used {} local \
                 objects)",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes(),
                stats.local_objects()
            );
        } else {
            println!(
                "\rReceived {}/{} objects in {} bytes",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes()
            );
        }
    
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        Ok(repo.reference_to_annotated_commit(&fetch_head)?)
    }
    

    pub fn has_conflicts(repo: &Repository, local: &git2::AnnotatedCommit, remote: &git2::AnnotatedCommit) -> Result<(), Box<dyn std::error::Error>> {
        let local_tree = repo.find_commit(local.id())?.tree()?;
        let remote_tree = repo.find_commit(remote.id())?.tree()?;
        let ancestor = repo
            .find_commit(repo.merge_base(local.id(), remote.id())?)?
            .tree()?;
        let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;
    
        if idx.has_conflicts() {
            println!("Merge conficts detected...");
            // repo.checkout_index(Some(&mut idx), None)?;
            return Err("error".into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    
        Ok(())
    }
}
