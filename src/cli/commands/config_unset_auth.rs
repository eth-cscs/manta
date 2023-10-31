use std::{fs, path::PathBuf};

use directories::ProjectDirs;

pub async fn exec() {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path_to_manta_authentication_token_file =
        PathBuf::from(project_dirs.unwrap().cache_dir());

    path_to_manta_authentication_token_file.push("http"); // ~/.config/manta/config is the file

    log::debug!(
        "Deleting manta authentication file {}",
        &path_to_manta_authentication_token_file.to_string_lossy()
    );

    let _ = fs::remove_file(path_to_manta_authentication_token_file);

    println!("Athentication token file deleted");
}
