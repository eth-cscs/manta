//! Table and JSON renderers for CFS configuration output.

use chrono::{DateTime, Local};
use comfy_table::Table;
use manta_shared::shared::dto::CfsConfigurationResponse;

use manta_shared::common::DATETIME_FORMAT;

/// Print CFS configurations as a formatted table.
pub fn print_table_struct(cfs_configurations: &[CfsConfigurationResponse]) {
  let mut table = Table::new();

  table.set_header(vec!["Config Name", "Last updated", "Layers"]);

  for cfs_configuration in cfs_configurations {
    let mut layers: String = String::new();

    if let Some(first_layer) = cfs_configuration.layers.first() {
      let layers_json = &cfs_configuration.layers;

      layers = format!(
        "Name:     {}\nPlaybook: {}\nCommit:   {}",
        first_layer.name.as_ref().unwrap_or(&String::new()),
        first_layer.playbook,
        first_layer.commit.as_deref().unwrap_or("Not defined"),
      );

      for layer in layers_json.iter().skip(1) {
        layers = format!(
          "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
          layers,
          layer.name.as_ref().unwrap_or(&String::new()),
          layer.playbook,
          layer.commit.as_deref().unwrap_or("Not defined"),
        );
      }
    }

    table.add_row(vec![
      cfs_configuration.name.clone(),
      cfs_configuration
        .last_updated
        .clone()
        .parse::<DateTime<Local>>()
        .map_or_else(
          |_| cfs_configuration.last_updated.clone(),
          |dt| dt.format(DATETIME_FORMAT).to_string(),
        ),
      layers,
    ]);
  }

  println!("{table}");
}
