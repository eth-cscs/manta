use crate::common::{self, app_context::AppContext};
use crate::service;
use anyhow::bail;
use chrono::NaiveDateTime;
use comfy_table::Table;

/// Delete CFS configurations and their derived artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  configuration_name_pattern_opt: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  assume_yes: bool,
) -> Result<(), anyhow::Error> {
  let candidates = service::configuration::get_deletion_candidates(
    &ctx.infra,
    token,
    ctx.cli.settings_hsm_group_name_opt,
    configuration_name_pattern_opt,
    since_opt,
    until_opt,
  )
  .await?;

  // Print summary tables
  println!("CFS sessions to delete:");
  let mut cfs_session_table = Table::new();
  cfs_session_table.set_header(vec!["Name", "Configuration", "Image ID"]);
  for cfs_session in &candidates.cfs_sessions_to_delete {
    cfs_session_table.add_row(vec![
      cfs_session.name.clone(),
      cfs_session.get_configuration_name().unwrap_or_default(),
      cfs_session.get_first_result_id().unwrap_or_default(),
    ]);
  }
  println!("{cfs_session_table}");

  println!("BOS sessiontemplates to delete:");
  let mut bos_sessiontemplate_table = Table::new();
  bos_sessiontemplate_table.set_header(vec![
    "Name",
    "Configuration",
    "Image ID",
  ]);
  for tuple in &candidates.bos_sessiontemplate_tuples {
    bos_sessiontemplate_table.add_row(vec![
      tuple.0.clone(),
      tuple.1.clone(),
      tuple.2.clone(),
    ]);
  }
  println!("{bos_sessiontemplate_table}");

  println!("CFS configurations to delete:");
  let mut cfs_configuration_table = Table::new();
  cfs_configuration_table.set_header(vec!["Name", "Last Update"]);
  for config in &candidates.configurations {
    cfs_configuration_table.add_row(vec![
      config.name.clone(),
      config.last_updated.clone(),
    ]);
  }
  println!("{cfs_configuration_table}");

  println!("Images to delete:");
  let mut image_id_table = Table::new();
  image_id_table.set_header(vec!["Image ID"]);
  for image_id in &candidates.image_ids {
    image_id_table.add_row(vec![image_id]);
  }
  println!("{image_id_table}");

  // Ask user for confirmation
  if !common::user_interaction::confirm(
    "Please review the data above and confirm to delete:",
    assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  // Execute deletion
  service::configuration::delete_configurations_and_derivatives(
    &ctx.infra,
    token,
    &candidates,
  )
  .await?;

  Ok(())
}
