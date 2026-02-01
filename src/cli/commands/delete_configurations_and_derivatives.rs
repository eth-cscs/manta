use crate::{
  common::{authentication::get_api_token, authorization::get_groups_names_available},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use chrono::NaiveDateTime;
use comfy_table::Table;

use crate::common;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  settings_hsm_group_name_opt: Option<&String>,
  configuration_name_pattern_opt: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  assume_yes: bool,
) -> Result<(), anyhow::Error> {
  if since_opt.is_some()
    && until_opt.is_some()
    && since_opt.unwrap() > until_opt.unwrap()
  {
    return Err(anyhow::Error::msg(
      "ERROR - 'since' date can't be after 'until' date. Exit".to_string(),
    ));
  }
  let shasta_token = get_api_token(backend, site_name).await?;
  let target_hsm_group_vec =
    if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
      vec![settings_hsm_group_name.clone()]
    } else {
      get_groups_names_available(
        backend,
        &shasta_token,
        None,
        settings_hsm_group_name_opt,
      )
      .await?
    };

  // COLLECT SITE WIDE DATA FOR VALIDATION
  //
  let (
    cfs_session_to_delete_vec,
    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec,
    image_id_vec,
    cfs_configuration_name_vec,
    cfs_session_cfs_configuration_image_id_tuple_filtered_vec,
    cfs_configuration_vec,
  ) = backend
    .get_data_to_delete(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
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
  if !common::user_interaction::confirm(
    "Please revew the data above and confirm to delete:",
    assume_yes,
  ) {
    return Err(anyhow::Error::msg(
      "Operation canceled by the user.".to_string(),
    ));
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
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &cfs_configuration_name_vec,
      &image_id_vec,
      &cfs_session_name_vec,
      &bos_sessiontemplate_name_vec,
    )
    .await?;

  Ok(())
}
