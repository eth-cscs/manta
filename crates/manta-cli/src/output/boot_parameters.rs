//! Renderer for [`BootParameters`] (the BSS boot-parameter
//! resource).
//!
//! Called by `manta get boot-parameters`. Supported output formats:
//! **JSON only** — the nested BSS payload (kernel, initrd, params,
//! cloud-init, ...) is too deep for a useful table, so the format
//! is pinned. The `-o` flag still defaults to `"table"` for the
//! subcommand, but the renderer ignores it and always emits JSON.

use anyhow::Error;

use crate::openapi_client::types::BootParameters;

/// Pretty-print boot parameters as JSON.
///
/// # Errors
///
/// Returns `Err` if `serde_json::to_string_pretty` fails (only
/// possible if the wire types stop being `Serialize`, which would
/// be a compile-time break).
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
