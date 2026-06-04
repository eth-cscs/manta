//! Table and JSON renderers for BSS boot parameter output.

use anyhow::Error;
use manta_shared::shared::dto::BootParameters;

/// Print boot parameters in the requested format.
///
/// Currently only JSON pretty-print is supported.
pub fn print(
  boot_parameters: &[BootParameters],
  _output_opt: Option<&str>,
) -> Result<(), Error> {
  println!("{}", serde_json::to_string_pretty(boot_parameters)?);
  Ok(())
}
