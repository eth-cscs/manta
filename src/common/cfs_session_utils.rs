use chrono::{DateTime, Local};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::{
  self, Group, cfs::session::CfsSessionGetResponse,
};

use super::DATETIME_FORMAT;

fn cfs_session_struct_to_vec(
  cfs_session: &manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse,
) -> Vec<String> {
  let start_time_utc = cfs_session
    .get_start_time()
    .and_then(|date_time| {
      let date = format!("{}Z", date_time);
      date.parse::<DateTime<Local>>().ok()
    })
    .unwrap_or_default();

  let completion_time_utc_opt =
    cfs_session.get_completion_time().and_then(|date_time| {
      let date = format!("{}Z", date_time);
      date.parse::<DateTime<Local>>().ok()
    });

  let status = if cfs_session.is_success() {
    "Succeeded".to_string()
  } else if completion_time_utc_opt.is_some() && !cfs_session.is_success() {
    "Failed".to_string()
  } else {
    cfs_session
      .status()
      .unwrap_or_else(|| "Unknown".to_string())
  };

  let duration_in_minutes =
    if let Some(completion_time_utc) = completion_time_utc_opt {
      let start_complete_diff = completion_time_utc - start_time_utc;
      (start_complete_diff.as_seconds_f64() / 60.0) as i64
    } else {
      let now_utc: DateTime<Local> = Local::now();
      let start_now_diff = now_utc - start_time_utc;
      (start_now_diff.as_seconds_f64() / 60.0) as i64
    };

  let mut result: Vec<String> = vec![cfs_session.name.clone()];
  result.push(
    cfs_session
      .configuration
      .as_ref()
      .and_then(|c| c.name.clone())
      .unwrap_or_default(),
  );
  result.push(start_time_utc.format(DATETIME_FORMAT).to_string());
  result.push(
    completion_time_utc_opt
      .map(|completion_time| {
        completion_time.format(DATETIME_FORMAT).to_string()
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

  let target = if cfs_session
    .target
    .as_ref()
    .and_then(|t| t.groups.as_ref())
    .map(|g| !g.is_empty())
    .unwrap_or(false)
  {
    let mut target_aux = cfs_session
      .target
      .as_ref()
      .and_then(|t| t.groups.as_ref())
      .map(|groups| {
        groups
          .iter()
          .map(|group| group.name.to_string())
          .collect::<Vec<String>>()
      })
      .unwrap_or_default();
    target_aux.sort();
    target_aux.join("\n")
  } else {
    let mut target_aux: Vec<String> = cfs_session
      .ansible
      .as_ref()
      .and_then(|a| a.limit.as_ref())
      .cloned()
      .unwrap_or_default()
      .split(',')
      .map(str::to_string)
      .collect();
    target_aux.sort();
    target_aux.join("\n")
  };
  result.push(target);
  result.push(
    cfs_session
      .status
      .as_ref()
      .and_then(|s| s.artifacts.as_ref())
      .and_then(|artifacts| artifacts.first())
      .and_then(|artifact| artifact.result_id.clone())
      .unwrap_or_default(),
  );

  result
}

/// Check if a CFS session targets any group the user has
/// access to.
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

/// Print CFS sessions as a formatted table.
pub fn print_table_struct(
  get_cfs_session_value_list: &[types::cfs::session::CfsSessionGetResponse],
) {
  let mut table = get_table_struct(get_cfs_session_value_list);
  table.set_content_arrangement(ContentArrangement::Dynamic);

  println!("{table}");
}

/// Build a table of CFS sessions without printing it.
pub fn get_table_struct(
  get_cfs_session_value_list: &[types::cfs::session::CfsSessionGetResponse],
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
    table.add_row(cfs_session_struct_to_vec(cfs_session_value));
  }

  table
}
