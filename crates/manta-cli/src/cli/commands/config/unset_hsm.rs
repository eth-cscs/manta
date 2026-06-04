//! Implements the `manta config unset hsm` command.

use anyhow::Error;

use crate::cli::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Remove the default HSM group from configuration.
pub fn exec() -> Result<(), Error> {
  unset_hsm()
}

fn unset_hsm() -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  tracing::info!("Unset HSM group");
  doc.remove("hsm_group");

  write_config_toml(&path, &doc)?;

  action_result::print("Target HSM group unset", None)?;

  Ok(())
}
