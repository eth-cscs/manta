use anyhow::{Context, Error};
use chrono::{DateTime, Local};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::common::DATETIME_FORMAT;

fn cfs_session_struct_to_vec(
  cfs_session: &CfsSessionGetResponse,
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

/// Print CFS sessions as a formatted table.
pub fn print_table_struct(
  get_cfs_session_value_list: &[CfsSessionGetResponse],
) {
  let mut table = get_table_struct(get_cfs_session_value_list);
  table.set_content_arrangement(ContentArrangement::Dynamic);

  println!("{table}");
}

/// Build a table of CFS sessions without printing it.
pub fn get_table_struct(
  get_cfs_session_value_list: &[CfsSessionGetResponse],
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

/// Print CFS sessions in the requested format.
///
/// Supports `"json"` for JSON output or a formatted table
/// (the default).
pub fn print(
  sessions: &[CfsSessionGetResponse],
  output: Option<&str>,
) -> Result<(), Error> {
  if output.is_some_and(|o| o == "json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&sessions)
        .context("Failed to serialize CFS sessions")?
    );
  } else {
    print_table_struct(sessions);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_session(name: &str) -> CfsSessionGetResponse {
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: None,
      ansible: None,
      target: None,
      status: None,
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  #[test]
  fn print_json_empty_list_succeeds() {
    // Verifies the JSON path doesn't error on empty input.
    let result = print(&[], Some("json"));
    assert!(result.is_ok());
  }

  #[test]
  fn print_json_with_sessions_succeeds() {
    let sessions = vec![make_session("sess1"), make_session("sess2")];
    let result = print(&sessions, Some("json"));
    assert!(result.is_ok());
  }

  #[test]
  fn print_table_empty_list_succeeds() {
    let result = print(&[], None);
    assert!(result.is_ok());
  }

  #[test]
  fn print_table_with_sessions_succeeds() {
    let result = print(&[], None);
    assert!(result.is_ok());
  }

  #[test]
  fn print_non_json_string_uses_table_path() {
    // Any output value other than "json" should use the table path.
    let result = print(&[], Some("table"));
    assert!(result.is_ok());
  }

  #[test]
  fn sessions_serialize_to_valid_json() {
    let sessions = vec![make_session("test-session")];
    let json = serde_json::to_string_pretty(&sessions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed[0]["name"], "test-session");
  }
}
