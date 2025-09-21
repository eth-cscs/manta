use chrono::{DateTime, Local};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::{self, cfs::session::CfsSessionGetResponse, Group};


pub fn cfs_session_struct_to_vec(
  cfs_session: manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse,
) -> Vec<String> {
  let start_time_utc = cfs_session
    .get_start_time()
    .map(|date_time| {
      let date = format!("{}Z", date_time);
      date.parse::<DateTime<Local>>().unwrap()
    })
    .unwrap_or_default();

  let completion_time_utc_opt =
    cfs_session.get_completion_time().map(|date_time| {
      let date = format!("{}Z", date_time);
      date.parse::<DateTime<Local>>().unwrap()
    });

  let status = if cfs_session.is_success() {
    "Succeeded".to_string()
  } else if completion_time_utc_opt.is_some() && !cfs_session.is_success() {
    "Failed".to_string()
  } else {
    cfs_session.status().unwrap_or("Unknown".to_string())
  };

  let duration_in_minutes =
    if let Some(completion_time_utc) = completion_time_utc_opt {
      let start_complete_diff = completion_time_utc - start_time_utc;
      let duration = (start_complete_diff.as_seconds_f64() / 60.0) as i64;
      duration
    } else {
      let now_utc: DateTime<Local> = Local::now();
      let start_now_diff = now_utc - start_time_utc;
      let duration = (start_now_diff.as_seconds_f64() / 60.0) as i64;
      duration
    };

  let mut result = vec![cfs_session.name.clone().unwrap()];
  result.push(cfs_session.configuration.clone().unwrap().name.unwrap());
  result.push(start_time_utc.format("%d/%m/%Y %H:%M:%S").to_string());
  result.push(
    completion_time_utc_opt
      .map(|completion_time| {
        completion_time.format("%d/%m/%Y %H:%M:%S").to_string()
      })
      .unwrap_or_default(),
  );
  result.push(duration_in_minutes.to_string());

  result.push(status);

  let session_type = match cfs_session.get_target_def().as_deref() {
    Some("dynamic") => "Runtime".to_string(),
    Some("image") => "Image".to_string(),
    Some(other) => other.to_string(),
    None => "Unknown".to_string(),
  };

  result.push(session_type);

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
  let mut table = get_table_struct(get_cfs_session_value_list);
  table.set_content_arrangement(ContentArrangement::Dynamic);

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
    "Completion",
    "Duration",
    "Status",
    "Type",
    "Target",
    "Image ID",
  ]);

  for cfs_session_value in get_cfs_session_value_list {
    table.add_row(cfs_session_struct_to_vec(cfs_session_value.clone()));
  }

  table
}

/* pub async fn get_image_id_related_to_cfs_configuration(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_name: &String,
) -> Option<String> {
  // Get all CFS sessions which has succeeded
  let cfs_sessions_list = backend
    .get_sessions(
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
      None,
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
} */

/* pub async fn get_image_id_from_cfs_session_and_cfs_configuration(
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
} */
