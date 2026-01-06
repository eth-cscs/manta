use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use chrono::NaiveDateTime;
use comfy_table::Table;
use dialoguer::{Confirm, theme::ColorfulTheme};
use manta_backend_dispatcher::{
  error::Error,
  interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait,
};

/* pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  target_hsm_group_vec: &[&str],
  // configuration_name_opt: Option<&String>,
  configuration_name_pattern: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  assume_yes: bool,
) -> Result<(), Error> {
  backend
    .i_delete_data_related_to_cfs_configuration(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_vec,
      // configuration_name_opt,
      configuration_name_pattern,
      since_opt,
      until_opt,
      assume_yes,
    )
    .await
} */

pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  target_hsm_group_vec: &[&str],
  configuration_name_pattern_opt: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  assume_yes: bool,
) -> Result<(), Error> {
  // COLLECT SITE WIDE DATA FOR VALIDATION
  //
  /* let (
    cfs_session_to_delete_vec,
    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec,
    image_id_vec,
    cfs_configuration_name_vec,
    cfs_session_cfs_configuration_image_id_tuple_filtered_vec,
    cfs_configuration_vec,
  ) = get_data_to_delete( */
  let (
    cfs_session_to_delete_vec,
    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec,
    image_id_vec,
    cfs_configuration_name_vec,
    cfs_session_cfs_configuration_image_id_tuple_filtered_vec,
    cfs_configuration_vec,
  ) = backend
    .get_data_to_delete(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_vec,
      configuration_name_pattern_opt,
      since_opt,
      until_opt,
    )
    .await?;

  // PRINT SUMMARY/DATA TO DELETE
  //
  println!("CFS sessions to delete:");

  let mut cfs_session_table = Table::new();

  cfs_session_table.set_header(vec!["Name", "Configuration", "Image ID"]);

  for cfs_session in &cfs_session_to_delete_vec {
    cfs_session_table.add_row(vec![
      cfs_session.name.as_ref().unwrap_or(&"".to_string()),
      &cfs_session
        .get_configuration_name()
        .unwrap_or_default()
        .to_string(),
      &cfs_session
        .get_first_result_id()
        .unwrap_or_default()
        .to_string(),
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

  for bos_sessiontemplate_tuple in
    &bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
  {
    bos_sessiontemplate_table.add_row(vec![
      bos_sessiontemplate_tuple.0.clone(),
      bos_sessiontemplate_tuple.1.clone(),
      bos_sessiontemplate_tuple.2.clone(),
    ]);
  }

  println!("{bos_sessiontemplate_table}");

  println!("CFS configurations to delete:");

  let mut cfs_configuration_table = Table::new();

  cfs_configuration_table.set_header(vec!["Name", "Last Update"]);

  for cfs_configuration_value in cfs_configuration_vec {
    cfs_configuration_table.add_row(vec![
      cfs_configuration_value.name,
      cfs_configuration_value.last_updated,
    ]);
  }

  println!("{cfs_configuration_table}");

  println!("Images to delete:");

  let mut image_id_table = Table::new();

  image_id_table.set_header(vec!["Image ID"]);

  for image_id in &image_id_vec {
    image_id_table.add_row(vec![image_id]);
  }

  println!("{image_id_table}");

  // ASK USER FOR CONFIRMATION
  //
  if !assume_yes {
    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("Please revew the data above and confirm to delete:")
      .interact()
      .unwrap()
    {
      println!("Continue");
    } else {
      println!("Cancelled by user. Aborting.");
      std::process::exit(0);
    }
  }

  // DELETE DATA
  //
  let cfs_session_name_vec: Vec<String> =
    cfs_session_cfs_configuration_image_id_tuple_filtered_vec
      .into_iter()
      .map(|(session, _, _)| session)
      .collect();

  let bos_sessiontemplate_name_vec: Vec<String> =
    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
      .into_iter()
      .map(|(sessiontemplate, _, _)| sessiontemplate)
      .collect();

  backend
    .delete(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &cfs_configuration_name_vec,
      &image_id_vec,
      &cfs_session_name_vec,
      &bos_sessiontemplate_name_vec,
    )
    .await
}
