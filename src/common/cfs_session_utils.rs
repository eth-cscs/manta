use chrono::{DateTime, Local};
use comfy_table::Table;
use manta_backend_dispatcher::{
  interfaces::{cfs::CfsTrait, ims::ImsTrait},
  types::{self, cfs::session::CfsSessionGetResponse, Group},
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub fn cfs_session_struct_to_vec(
  cfs_session: manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse,
) -> Vec<String> {
  let start_time_utc_str = cfs_session
    .get_start_time()
    .and_then(|date_time| Some(date_time.to_string() + "Z"))
    .unwrap_or("".to_string());

  let mut result = vec![cfs_session.name.clone().unwrap()];
  result.push(cfs_session.configuration.clone().unwrap().name.unwrap());
  result.push(
    start_time_utc_str
      .parse::<DateTime<Local>>()
      .unwrap()
      .format("%d/%m/%Y %H:%M:%S")
      .to_string(),
  );

  result.push(
    cfs_session
      .status
      .as_ref()
      .unwrap()
      .session
      .as_ref()
      .unwrap()
      .status
      .as_ref()
      .unwrap_or(&"".to_string())
      .to_string(),
  );
  result.push(cfs_session.is_success().to_string());
  result.push(
    cfs_session
      .get_target_def()
      .unwrap_or("".to_string())
      .to_string(),
  );
  let target = if !cfs_session
    .target
    .as_ref()
    .unwrap()
    .groups
    .as_ref()
    .unwrap_or(&Vec::new())
    .is_empty()
  {
    let mut target_aux = cfs_session
      .target
      .as_ref()
      .unwrap()
      .groups
      .as_ref()
      .unwrap()
      .iter()
      .map(|group| group.name.to_string())
      .collect::<Vec<String>>();
    target_aux.sort();
    target_aux.join("\n")
  } else {
    let mut target_aux: Vec<String> = cfs_session
      .ansible
      .as_ref()
      .unwrap()
      .limit
      .as_ref()
      .cloned()
      .unwrap_or_default()
      .split(',')
      .map(|xname| xname.to_string())
      .collect();
    target_aux.sort();
    target_aux.join("\n")
  };
  result.push(target);
  result.push(
    cfs_session
      .status
      .unwrap()
      .artifacts
      .unwrap_or_default()
      .first()
      .and_then(|artifact| artifact.result_id.clone())
      .unwrap_or("".to_string()),
  );

  result
}

// Check if a session is related to a group the user has access to
pub fn check_cfs_session_against_groups_available(
  cfs_session: &CfsSessionGetResponse,
  group_available: Vec<Group>,
) -> bool {
  group_available.iter().any(|group| {
    cfs_session
      .get_target_hsm()
      .is_some_and(|group_vec| group_vec.contains(&group.label))
      || cfs_session
        .get_target_xname()
        .is_some_and(|session_xname_vec| {
          session_xname_vec
            .iter()
            .all(|xname| group.get_members().contains(xname))
        })
  })
}

pub fn print_table_struct(
  get_cfs_session_value_list: &Vec<types::cfs::session::CfsSessionGetResponse>,
) {
  let table = get_table_struct(get_cfs_session_value_list);

  println!("{table}");
}

pub fn get_table_struct(
  get_cfs_session_value_list: &Vec<types::cfs::session::CfsSessionGetResponse>,
) -> Table {
  let mut table = Table::new();

  table.set_header(vec![
    "Session Name",
    "Configuration Name",
    "Start",
    "Status",
    "Succeeded",
    "Target Def",
    "Target",
    "Image ID",
  ]);

  for cfs_session_value in get_cfs_session_value_list {
    table.add_row(cfs_session_struct_to_vec(cfs_session_value.clone()));
  }

  table
}

pub async fn get_image_id_related_to_cfs_configuration(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_name: &String,
) -> Option<String> {
  // Get all CFS sessions which has succeeded
  let cfs_sessions_list = backend
    .get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      Some(true),
    )
    .await
    .unwrap();

  get_image_id_from_cfs_session_and_cfs_configuration(
    backend,
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    cfs_configuration_name,
    &cfs_sessions_list,
  )
  .await
}

pub async fn get_image_id_from_cfs_session_and_cfs_configuration(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_name: &String,
  cfs_sessions_vec: &[CfsSessionGetResponse],
) -> Option<String> {
  // Filter CFS sessions to the ones related to CFS configuration and built an image (target
  // definition is 'image' and it actually has at least one artifact)
  let cfs_session_image_succeeded =
    cfs_sessions_vec.iter().filter(|cfs_session| {
      cfs_session
        .get_configuration_name()
        .unwrap()
        .eq(cfs_configuration_name)
        && cfs_session.get_target_def().unwrap().eq("image")
        && cfs_session.get_first_result_id().is_some()
    });

  // Find image in CFS sessions
  for cfs_session in cfs_session_image_succeeded {
    let cfs_session_name = cfs_session.name.as_ref().unwrap();

    let image_id = cfs_session.get_first_result_id().unwrap();

    log::info!(
      "Checking if result_id {} in CFS session {} exists",
      image_id,
      cfs_session_name
    );

    // Get IMS image related to the CFS session
    let image_vec_rslt = backend
      .get_images(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&image_id),
      )
      .await;

    if image_vec_rslt.is_ok() {
      log::info!(
        "Found the image ID '{}' related to CFS sesison '{}'",
        image_id,
        cfs_session_name,
      );

      return Some(image_id.to_string()); // from https://users.rust-lang.org/t/convert-option-str-to-option-string/20533/2
    };
  }

  None
}
