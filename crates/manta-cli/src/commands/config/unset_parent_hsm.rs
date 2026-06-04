//! Implements the `manta config unset parent-hsm` command.

use anyhow::Error;

use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Remove the parent HSM group from configuration.
///
/// Pure local-file edit — no backend or server interaction.
pub fn exec() -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;
  tracing::info!("Unset parent HSM group");
  doc.remove("parent_hsm_group");
  write_config_toml(&path, &doc)?;
  action_result::print("Parent HSM group unset", None)?;
  Ok(())
}
