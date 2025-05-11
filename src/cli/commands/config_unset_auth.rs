use std::{fs, path::PathBuf};

use dialoguer::Select;
use directories::ProjectDirs;

pub async fn exec() {
  let mut auth_token_list: Vec<PathBuf> = vec![];

  // XDG Base Directory Specification
  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let path_to_manta_authentication_token_file =
    PathBuf::from(project_dirs.unwrap().cache_dir());

  for entry in fs::read_dir(path_to_manta_authentication_token_file).unwrap() {
    auth_token_list.push(entry.unwrap().path())
  }

  let selection = Select::new()
    .with_prompt("Please choose the site token to delete from the list below")
    .default(0)
    .items(
      &auth_token_list
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<_>>(),
    )
    .interact()
    .unwrap();

  println!(
    "Deleting authentication file: {}",
    auth_token_list[selection]
      .file_name()
      .unwrap()
      .to_str()
      .unwrap()
  );

  fs::remove_file(auth_token_list[selection].clone()).unwrap();

  /* for auth_token in auth_token_list {
      log::debug!(
          "Deleting manta authentication file {}",
          &auth_token.to_string_lossy()
      );
      fs::remove_file(auth_token).unwrap();
  }

  println!("Athentication token file deleted"); */
}
