use chrono::{DateTime, Local};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::{
  bos::session_template::BosSessionTemplate,
  cfs::cfs_configuration_details::ConfigurationDetails,
  cfs::cfs_configuration_response::CfsConfigurationResponse,
  cfs::session::CfsSessionGetResponse, ims::Image,
};

use super::DATETIME_FORMAT;

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
        first_layer.name,
        first_layer.playbook,
        first_layer.commit.as_deref().unwrap_or("Not defined"),
      );

      for layer in layers_json.iter().skip(1) {
        layers = format!(
          "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
          layers,
          layer.name,
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
        .map(|dt| dt.format(DATETIME_FORMAT).to_string())
        .unwrap_or_else(|_| cfs_configuration.last_updated.clone()),
      layers,
    ]);
  }

  println!("{table}");
}

/// Print a detailed CFS configuration with its derived
/// sessions, templates, and images.
pub fn print_table_details_struct(
  cfs_configuration: ConfigurationDetails,
  cfs_session_vec_opt: Option<Vec<CfsSessionGetResponse>>,
  bos_sessiontemplate_vec_opt: Option<Vec<BosSessionTemplate>>,
  image_vec_opt: Option<Vec<Image>>,
) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "Configuration Name",
    "Last updated",
    "Layers",
    "Derivatives",
  ]);

  let mut layers: String = String::new();

  for layer in cfs_configuration.config_layers {
    layers = format!(
      "{}\n\nName:     {}\nBranch:   {}\nTag:      {}\nDate:     {}\nAuthor:   {}\nCommit:   {}\nPlaybook: {}",
      layers,
      layer.name,
      layer.branch,
      layer.tag,
      layer.commit_date,
      layer.author,
      layer.commit_id,
      layer.playbook
    );
  }

  let mut derivatives: String = String::new();

  if let Some(cfs_session_vec) = cfs_session_vec_opt {
    derivatives += "CFS sessions:";
    for cfs_session in cfs_session_vec {
      derivatives = derivatives + "\n - " + &cfs_session.name;
    }
  }

  if let Some(bos_sessiontemplate_vec) = bos_sessiontemplate_vec_opt {
    derivatives += "\n\nBOS sessiontemplates:";
    for bos_sessiontemplate in bos_sessiontemplate_vec {
      derivatives =
        derivatives + "\n - " + &bos_sessiontemplate.name.unwrap_or_default();
    }
  }

  if let Some(image_vec) = image_vec_opt {
    derivatives += "\n\nIMS images:";
    for image in image_vec {
      derivatives = derivatives + "\n - " + &image.name;
    }
  }

  layers = layers.trim_start_matches("\n\n").to_string();

  table.add_row(vec![
    cfs_configuration.name,
    cfs_configuration
      .last_updated
      .parse::<DateTime<Local>>()
      .map(|dt| dt.format(DATETIME_FORMAT).to_string())
      .unwrap_or(cfs_configuration.last_updated),
    layers,
    derivatives,
  ]);

  println!("{table}");
}
