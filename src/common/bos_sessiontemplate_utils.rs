use comfy_table::Table;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::common;

pub fn print_table_struct(bos_sessiontemplate_vec: Vec<BosSessionTemplate>) {
  let mut table = Table::new();

  table.set_header(vec![
    "Name",
    "Image ID",
    "Runtime Configuration",
    "Cfs Enabled",
    "Target",
    "Compute Etag",
  ]);

  for bos_template in bos_sessiontemplate_vec {
    let enable_cfs = bos_template
      .enable_cfs
      .map(|value| value.to_string())
      .unwrap_or("N/A".to_string());

    for boot_set in bos_template.boot_sets.unwrap() {
      let target: Vec<String> = if boot_set.1.node_groups.is_some() {
        // NOTE: very
        // important to
        // define target
        // variable type to
        // tell compiler we
        // want a long live
        // variable
        boot_set.1.node_groups.unwrap()
      } else if boot_set.1.node_list.is_some() {
        boot_set.1.node_list.unwrap()
      } else {
        Vec::new()
      };

      table.add_row(vec![
        bos_template.name.clone().unwrap(),
        boot_set
          .1
          .path
          .unwrap()
          .trim_start_matches("s3://boot-images/")
          .trim_end_matches("/manifest.json")
          .to_string(),
        bos_template.cfs.clone().unwrap().configuration.unwrap(),
        enable_cfs.clone(),
        common::node_ops::string_vec_to_multi_line_string(Some(&target), 2),
        boot_set.1.etag.unwrap_or("".to_string()),
      ]);
    }
  }

  println!("{table}");
}
