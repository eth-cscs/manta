//! JSON renderer for BSS boot parameter output. There is no table
//! variant yet (the BSS payload is too nested for a useful table);
//! callers always get pretty-printed JSON.

use anyhow::Error;
use manta_shared::types::dto::BootParameters;

/// Pretty-print boot parameters as JSON.
pub fn print(boot_parameters: &[BootParameters]) -> Result<(), Error> {
  println!("{}", serde_json::to_string_pretty(boot_parameters)?);
  Ok(())
}
