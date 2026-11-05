//! Implements the `manta config unset auth` command.
//!
//! Deletes the cached token file (`<cache_dir>/token.json`). The next
//! command that needs a token runs the interactive login again.
//!
//! Legacy per-site `<site>_auth` files from older manta versions are
//! left alone: nothing reads them any more, and other manta versions
//! may still use them.

use anyhow::{Context, Error};

use crate::common::authentication::token_cache_path;
use crate::output::action_result;

/// Remove the cached authentication token.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be resolved, no
/// cached token is present, or the file cannot be removed.
pub fn exec() -> Result<(), Error> {
  let path = token_cache_path()?;

  if !path.exists() {
    anyhow::bail!("No cached authentication token found");
  }

  std::fs::remove_file(&path)
    .with_context(|| format!("Failed to delete '{}'", path.display()))?;

  action_result::print(
    &format!("Deleted cached authentication token: {}", path.display()),
    None,
  )?;

  Ok(())
}
