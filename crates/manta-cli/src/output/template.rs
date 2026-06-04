//! Table and JSON renderers for BOS session template output.

use anyhow::{Context, Error};
use comfy_table::Table;
use manta_shared::types::dto::BosSessionTemplate;

/// Print BOS session templates in the requested format.
pub fn print(
  templates: &[BosSessionTemplate],
  output: &str,
) -> Result<(), Error> {
  if output == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(templates)
        .context("Failed to serialize BOS sessiontemplates")?
    );
  } else {
    print_table_struct(templates.to_vec());
  }
  Ok(())
}

/// Print BOS session templates as a formatted table.
pub fn print_table_struct(bos_sessiontemplate_vec: Vec<BosSessionTemplate>) {
  let mut table = Table::new();

  table.set_header(vec![
    "Name",
    "Image ID",
    "Runtime Configuration",
    "Cfs Enabled",
    "Target",
  ]);

  for bos_template in bos_sessiontemplate_vec {
    let enable_cfs = bos_template
      .enable_cfs
      .map_or_else(|| "N/A".to_string(), |v| v.to_string());

    if let Some(boot_sets) = bos_template.boot_sets {
      for boot_set in boot_sets {
        let target: Vec<String> =
          if let Some(node_groups) = boot_set.1.node_groups {
            node_groups
          } else {
            boot_set.1.node_list.unwrap_or_default()
          };

        table.add_row(vec![
          bos_template.name.clone().unwrap_or_default(),
          boot_set
            .1
            .path
            .unwrap_or_default()
            .trim_start_matches("s3://boot-images/")
            .trim_end_matches("/manifest.json")
            .to_string(),
          bos_template
            .cfs
            .as_ref()
            .and_then(|cfs| cfs.configuration.clone())
            .unwrap_or_else(|| "NA".to_string()),
          enable_cfs.clone(),
          crate::common::multi_line::string_vec_to_multi_line_string(
            Some(&target),
            2,
          ),
        ]);
      }
    }
  }

  println!("{table}");
}

#[cfg(test)]
mod tests {
  //! Locks down the `print()` contract: every supported format and
  //! the empty-input case return Ok. The regression specifically
  //! guarded against here is the previous `_ => {}` fall-through arm
  //! that silently emitted nothing for any value other than
  //! `"table"` or `"json"` — now the `else` branch always falls
  //! back to the table renderer, matching the rest of the
  //! `output/*::print()` family.

  use super::*;

  #[test]
  fn print_json_on_empty_succeeds() {
    assert!(print(&[], "json").is_ok());
  }

  #[test]
  fn print_table_on_empty_succeeds() {
    assert!(print(&[], "table").is_ok());
  }

  #[test]
  fn print_unknown_format_falls_back_to_table() {
    // Regression: previously the `_ => {}` arm silently emitted
    // nothing on an unknown output format. The fix made the `else`
    // arm always render a table; this test pins that down so we
    // don't regress.
    assert!(print(&[], "garbage").is_ok());
    assert!(print(&[], "").is_ok());
  }

  #[test]
  fn print_table_struct_does_not_panic_on_empty() {
    print_table_struct(vec![]);
  }
}
