use anyhow::Error;

use crate::common::config::{read_config_toml, write_config_toml};

pub fn exec() -> Result<(), Error> {
  unset_hsm()
}

fn unset_hsm() -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  log::info!("Unset HSM group");
  doc.remove("hsm_group");

  write_config_toml(&path, &doc)?;

  println!("Target HSM group unset");

  Ok(())
}
