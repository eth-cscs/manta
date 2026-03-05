use comfy_table::Table;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::common;

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
      .map(|v| v.to_string())
      .unwrap_or_else(|| "N/A".to_string());

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
          common::node_ops::string_vec_to_multi_line_string(Some(&target), 2),
        ]);
      }
    }
  }

  println!("{table}");
}
