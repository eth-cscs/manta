use std::fs;

use anyhow::{Context, Error};
use dialoguer::Select;

use crate::common::config::get_default_cache_path;

/// Remove cached authentication credentials.
pub fn exec() -> Result<(), Error> {
  unset_auth()
}

fn unset_auth() -> Result<(), Error> {
  let mut auth_token_list: Vec<std::path::PathBuf> = vec![];

  let path_to_manta_authentication_token_file = get_default_cache_path()?;

  for entry in fs::read_dir(&path_to_manta_authentication_token_file)
    .context("Failed to read authentication token directory")?
  {
    auth_token_list.push(entry.context("Failed to read entry")?.path())
  }

  let selection = Select::new()
    .with_prompt("Please choose the site token to delete from the list below")
    .default(0)
    .items(
      auth_token_list
        .iter()
        .map(|path| {
          path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
        })
        .collect::<Vec<_>>(),
    )
    .interact()
    .context("Failed to get user selection")?;

  println!(
    "Deleting authentication file: {}",
    auth_token_list[selection]
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("unknown")
  );

  fs::remove_file(auth_token_list[selection].clone())?;

  Ok(())
}
