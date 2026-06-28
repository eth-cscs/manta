//! Implements the `manta config unset auth` command.
//!
//! Lists the cached per-site token files under the manta cache
//! directory, prompts the user to pick one, and deletes it. The next
//! invocation that targets that site will re-run the device-code login
//! flow.

use std::fs;

use anyhow::{Context, Error};
use dialoguer::Select;

use crate::output::action_result;
use manta_shared::common::config::get_default_cache_path;

/// Remove cached authentication credentials.
///
/// Interactive: prompts via `dialoguer::Select`; not suitable for
/// non-TTY contexts.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be read, no cached
/// tokens are present, the interactive prompt fails, or the file
/// cannot be removed.
pub fn exec() -> Result<(), Error> {
  unset_auth()
}

fn unset_auth() -> Result<(), Error> {
  let mut auth_token_list: Vec<std::path::PathBuf> = vec![];

  let path_to_manta_authentication_token_file = get_default_cache_path()?;

  for entry in fs::read_dir(&path_to_manta_authentication_token_file)
    .context("Failed to read authentication token directory")?
  {
    auth_token_list.push(entry.context("Failed to read entry")?.path());
  }

  if auth_token_list.is_empty() {
    anyhow::bail!("No cached authentication tokens found");
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

  action_result::print(
    &format!(
      "Deleting authentication file: {}",
      auth_token_list[selection]
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
    ),
    None,
  )?;

  fs::remove_file(auth_token_list[selection].clone())?;

  Ok(())
}
