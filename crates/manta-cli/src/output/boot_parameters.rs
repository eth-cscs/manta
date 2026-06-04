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

#[cfg(test)]
mod tests {
  //! Pins the JSON-only contract: empty input still produces valid
  //! JSON (`[]`), and the renderer never panics. Catches the kind of
  //! signature regression we hit before (the `output_opt` parameter
  //! used to be threaded in but ignored — it has since been dropped).

  use super::*;

  #[test]
  fn print_empty_succeeds() {
    assert!(print(&[]).is_ok());
  }

  #[test]
  fn empty_input_serializes_to_empty_json_array() {
    let json = serde_json::to_string(&Vec::<BootParameters>::new()).unwrap();
    assert_eq!(json, "[]");
  }
}
