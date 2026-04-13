use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::Context;
use comfy_table::{Cell, ContentArrangement, Table};
use csm_rs::node::types::NodeDetails;
use hostlist_parser::parse;
use manta_backend_dispatcher::{
  interfaces::hsm::{component::ComponentTrait, group::GroupTrait},
  types::Component,
};
use regex::Regex;

// Compile-time constant pattern — .expect() is safe here because
// the regex literal is known to be valid and will never fail.
static XNAME_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^x\d{4}c[0-7]s([0-9]|[1-5][0-9]|6[0-4])b[0-1]n[0-7]$")
    .expect("Invalid xname regex pattern")
});

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Length of a NID string, e.g. "nid000001" = 9 characters.
const NID_STRING_LENGTH: usize = 9;

/// Length of the xname blade prefix, e.g. "x1000c7s0b" = 10 characters.
const XNAME_BLADE_PREFIX_LEN: usize = 10;

// Validate and get short nid
fn get_short_nid(long_nid: &str) -> Result<usize, anyhow::Error> {
  // Validate nid has the right length
  if long_nid.len() != NID_STRING_LENGTH {
    anyhow::bail!(
      "Nid '{}' not valid, Nid does not have {} characters",
      long_nid,
      NID_STRING_LENGTH
    );
  }

  long_nid
    .strip_prefix("nid")
    .ok_or_else(|| {
      anyhow::anyhow!("Nid '{}' not valid, 'nid' prefix missing", long_nid)
    })
    .and_then(|nid_number| {
      nid_number.to_string().parse::<usize>().with_context(|| {
        format!(
          "Could not convert Nid '{}' from long to \
             short format",
          nid_number
        )
      })
    })
}

/// Resolve a NID hostlist expression to xnames by
/// cross-referencing available node metadata.
pub async fn get_xname_from_nid_hostlist(
  node_vec: &[String],
  node_metadata_available_vec: &[Component],
) -> Result<Vec<String>, anyhow::Error> {
  // Convert long nids to short nids
  // Get xnames from short nids
  let short_nid_vec: Vec<usize> = node_vec
    .iter()
    .map(|nid_long| get_short_nid(nid_long))
    .collect::<Result<Vec<_>, _>>()?;

  log::debug!("short Nid list expanded: {:?}", short_nid_vec);

  let xname_vec: Vec<String> = node_metadata_available_vec
    .iter()
    .filter(|node_metadata_available| {
      node_metadata_available
        .nid
        .is_some_and(|nid| short_nid_vec.contains(&nid))
    })
    .filter_map(|node_metadata_available| {
      node_metadata_available.id.as_ref().cloned()
    })
    .collect();

  Ok(xname_vec)
}

/// Filter available node metadata to only those xnames
/// present in `node_vec`.
pub async fn get_xname_from_xname_hostlist(
  node_vec: &[String],
  node_metadata_available_vec: &[Component],
) -> Result<Vec<String>, anyhow::Error> {
  // If hostlist of XNAMEs, return hostlist expanded xnames
  // Validate XNAMEs
  log::debug!("XNAME format are valid");

  let xname_vec: Vec<String> = node_metadata_available_vec
    .iter()
    .filter(|node_metadata_available| {
      node_metadata_available
        .id
        .as_ref()
        .is_some_and(|id| node_vec.contains(id))
    })
    .filter_map(|node_metadata_available| {
      node_metadata_available.id.as_ref().cloned()
    })
    .collect();

  Ok(xname_vec)
}

// Unused get_xname_from_nid_regex removed

// Unused get_xname_from_xname_regex removed

/// Convenience wrapper that fetches node metadata from the backend
/// and resolves a hosts expression to a sorted, deduplicated list
/// of xnames.
///
/// This combines the two-step pattern of
/// `backend.get_node_metadata_available()` followed by
/// `from_hosts_expression_to_xname_vec()` that appears in many
/// command files.
pub async fn resolve_hosts_expression(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hosts_expression: &str,
  is_include_siblings: bool,
) -> Result<Vec<String>, anyhow::Error> {
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .context("Could not get node metadata")?;

  let mut xname_vec = from_hosts_expression_to_xname_vec(
    hosts_expression,
    is_include_siblings,
    node_metadata_available_vec,
  )
  .await
  .context("Could not convert user input to list of xnames")?;

  xname_vec.sort();
  xname_vec.dedup();

  Ok(xname_vec)
}

/// Translates and filters a 'host expression' into a list of xnames.
/// a host expression is a comma separated list of NIDs or XNAMEs, a regex or a hostlist
/// NOTE: user can provice a host expression and expand the list to all siblings
pub async fn from_hosts_expression_to_xname_vec(
  user_input: &str,
  is_include_siblings: bool,
  node_metadata_available_vec: Vec<Component>,
) -> Result<Vec<String>, anyhow::Error> {
  // Check if hostlist
  // Expand user input to list of nids
  let hostlist_expanded_vec_rslt =
    parse(user_input).map_err(|e| anyhow::anyhow!(e.to_string()));

  let xname_vec = match hostlist_expanded_vec_rslt {
    Ok(node_vec) => {
    log::debug!("Hostlist format is valid");
    // If hostlist, expand hostlist
    let xname_vec: Vec<String> = if validate_nid_format_vec(&node_vec) {
      // If hostlist of NIDs, convert to xname
      // Validate NIDs
      log::debug!("NID format is valid");
      log::debug!("hostlist Nids: {}", user_input);
      log::debug!("hostlist Nids expanded: {:?}", node_vec);

      get_xname_from_nid_hostlist(&node_vec, &node_metadata_available_vec)
        .await?
    } else if validate_xname_format_vec(&node_vec) {
      // If hostlist of XNAMEs, return hostlist expanded xnames
      // Validate XNAMEs
      log::debug!("NID format is valid");
      log::debug!("hostlist Nids: {}", user_input);
      log::debug!("hostlist Nids expanded: {:?}", node_vec);

      get_xname_from_xname_hostlist(&node_vec, &node_metadata_available_vec)
        .await?
    } else {
      anyhow::bail!(
        "Could not parse user input as a list of nodes from a hostlist expression."
      );
    };

    xname_vec
    }
    Err(e) => {
      anyhow::bail!(
        "Could not parse user input as a list of nodes from a hostlist or regex expression: {e}"
      );
    }
  };

  if xname_vec.is_empty() {
    anyhow::bail!(
      "Could not parse user input as a list of nodes from a hostlist or regex expression."
    );
  }

  // Include siblings if requested
  let xname_vec: Vec<String> = if is_include_siblings {
    log::debug!("Include siblings");
    let xname_blade_vec: Vec<String> = xname_vec
      .iter()
      .map(|xname| xname.get(0..XNAME_BLADE_PREFIX_LEN).unwrap_or(xname).to_string())
      .collect();

    log::debug!("XNAME blades:\n{:?}", xname_blade_vec);

    // Filter xnames to the ones the user has access to

    node_metadata_available_vec
      .into_iter()
      .filter(|node_metadata_available| {
        node_metadata_available.id.as_ref().is_some_and(|id| {
          xname_blade_vec
            .iter()
            .any(|xname_blade| id.starts_with(xname_blade))
        })
      })
      .filter_map(|node_metadata_available| node_metadata_available.id)
      .collect()
  } else {
    xname_vec
  };

  Ok(xname_vec)
}

/// Returns a HashMap with keys HSM group names the user has access to and values a curated list of memembers that matches
/// hostlist
pub async fn get_curated_hsm_group_from_xname_hostlist(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  xname_vec: &[String],
) -> Result<HashMap<String, Vec<String>>, anyhow::Error> {
  // Create a summary of HSM groups and the list of members filtered by the list of nodes the
  // user is targeting
  let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

  let hsm_name_available_vec = backend
    .get_group_name_available(auth_token)
    .await
    .context("Failed to get available HSM group names")?;

  // Get HSM group user has access to
  let hsm_group_available_map = backend
    .get_group_map_and_filter_by_group_vec(
      auth_token,
      &hsm_name_available_vec
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>(),
    )
    .await
    .context("Failed to get HSM group summary")?;

  // Filter hsm group members
  for (hsm_name, hsm_members) in hsm_group_available_map {
    let xname_filtered: Vec<String> = hsm_members
      .iter()
      .filter(|&xname| xname_vec.contains(xname))
      .cloned()
      .collect();
    if !xname_filtered.is_empty() {
      hsm_group_summary.insert(hsm_name, xname_filtered);
    }
  }

  Ok(hsm_group_summary)
}

/// Check if input is a NID
fn validate_nid_format_vec(node_vec: &[String]) -> bool {
  node_vec.iter().all(|nid| validate_nid_format(nid))
}

/// Check if input is a NID
fn validate_nid_format(nid: &str) -> bool {
  nid.to_lowercase().starts_with("nid")
    && nid.len() == 9
    && nid
      .strip_prefix("nid")
      .is_some_and(|nid_number| nid_number.chars().all(char::is_numeric))
}

/// Validate xname is correct (it uses regex taken from HPE Cray CSM docs)
fn validate_xname_format_vec(node_vec: &[String]) -> bool {
  node_vec.iter().all(|nid| validate_xname_format(nid))
}

/// Validate xname is correct (it uses regex taken from HPE Cray CSM docs)
pub fn validate_xname_format(xname: &str) -> bool {
  XNAME_RE.is_match(xname)
}

/// Print a formatted table of node details. When `wide`
/// is true, an extra column for kernel parameters is shown.
pub fn print_table(nodes_status: Vec<NodeDetails>, wide: bool) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  let mut header = vec![
    "XNAME",
    "NID",
    "HSM",
    "Power",
    "Runtime Config",
    "Config Status",
    "Enabled",
    "Error #",
    "Image ID",
  ];

  if wide {
    header.push("Kernel Params");
  }

  table.set_header(header);

  for node_status in nodes_status {
    let mut node_vec: Vec<String> = node_status
      .hsm
      .split(',')
      .map(|xname_str| xname_str.trim().to_string())
      .collect();
    node_vec.sort();

    let mut row = vec![
      Cell::new(node_status.xname),
      Cell::new(node_status.nid),
      Cell::new(string_vec_to_multi_line_string(Some(&node_vec), 1)),
      Cell::new(node_status.power_status),
      Cell::new(node_status.desired_configuration),
      Cell::new(node_status.configuration_status),
      Cell::new(node_status.enabled),
      Cell::new(node_status.error_count),
      Cell::new(node_status.boot_image_id),
    ];

    if wide {
      let kernel_params_vec: Vec<&str> =
        node_status.kernel_params.split_whitespace().collect();
      let cell_max_width = kernel_params_vec
        .iter()
        .map(|value| value.len())
        .max()
        .unwrap_or(0);

      let mut kernel_params_string: String = kernel_params_vec
        .first()
        .map(|s| s.to_string())
        .unwrap_or_default();
      let mut cell_width = kernel_params_string.len();

      for kernel_param in kernel_params_vec.iter().skip(1) {
        cell_width += kernel_param.len();

        if cell_width + kernel_param.len() >= cell_max_width {
          kernel_params_string.push('\n');
          cell_width = 0;
        } else {
          kernel_params_string.push(' ');
        }

        kernel_params_string.push_str(kernel_param);
      }

      row.push(Cell::new(kernel_params_string));
    }

    table.add_row(row);
  }

  println!("{table}");
}

/// Print a two-column summary table from a counter hashmap.
fn print_counter_table(
  header: &str,
  value_header: &str,
  counters: &HashMap<String, usize>,
) {
  let mut table = Table::new();
  table.set_header(vec![header, value_header]);
  for (name, count) in counters {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(name),
        Cell::new(count).set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }
  println!("{table}");
}

/// Print aggregate summary tables showing counts by power
/// status, boot config, runtime config, and boot image.
pub fn print_summary(node_details_list: Vec<NodeDetails>) {
  let mut power_status_counters: HashMap<String, usize> = HashMap::new();
  let mut boot_configuration_counters: HashMap<String, usize> = HashMap::new();
  let mut runtime_configuration_counters: HashMap<String, usize> =
    HashMap::new();
  let mut boot_image_counters: HashMap<String, usize> = HashMap::new();

  for node in node_details_list {
    power_status_counters
      .entry(node.power_status)
      .and_modify(|power_status_counter| *power_status_counter += 1)
      .or_insert(1);

    boot_configuration_counters
      .entry(node.boot_configuration)
      .and_modify(|power_status_counter| *power_status_counter += 1)
      .or_insert(1);

    runtime_configuration_counters
      .entry(node.desired_configuration)
      .and_modify(|power_status_counter| *power_status_counter += 1)
      .or_insert(1);

    boot_image_counters
      .entry(node.boot_image_id)
      .and_modify(|power_status_counter| *power_status_counter += 1)
      .or_insert(1);
  }

  let mut table = Table::new();

  table.set_header(vec!["Power status", "Num nodes"]);

  for power_status in
    ["FAILED", "ON", "OFF", "READY", "STANDBY", "UNCONFIGURED"]
  {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(power_status),
        Cell::new(power_status_counters.get(power_status).unwrap_or(&0))
          .set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }

  println!("{table}");

  print_counter_table(
    "Boot configuration name",
    "Num nodes",
    &boot_configuration_counters,
  );

  print_counter_table("Boot image id", "Num nodes", &boot_image_counters);

  print_counter_table(
    "Runtime configuration name",
    "Num nodes",
    &runtime_configuration_counters,
  );
}

/// Format a slice of strings into comma-separated lines,
/// wrapping after every `num_columns` entries.
pub fn string_vec_to_multi_line_string(
  nodes: Option<&[String]>,
  num_columns: usize,
) -> String {
  if num_columns == 0 {
    return String::new();
  }

  let mut members: String;

  match nodes {
    Some(nodes) if !nodes.is_empty() => {
      // Safe: guarded by !is_empty()
      members = nodes[0].to_string();

      for (i, node) in nodes.iter().enumerate().skip(1) {
        // iterate for the rest of the list
        if i % num_columns == 0 {
          // breaking the cell content into multiple lines (only 2 xnames per line)

          members.push_str(",\n");
        } else {
          members.push(',');
        }

        members.push_str(node);
      }
    }
    _ => members = String::new(),
  }

  members
}

/// Resolve target nodes from either a hosts expression, an
/// explicit HSM group name, or the settings-level HSM group.
///
/// Priority order:
/// 1. `hosts_expression` — parsed and validated via
///    [`resolve_hosts_expression`].
/// 2. `hsm_group_name_arg_opt` — the CLI `--hsm-group`
///    argument; validated for access via
///    [`get_groups_names_available`], then expanded to member
///    xnames.
/// 3. `settings_hsm_group_name_opt` — the group configured in
///    the environment or config file; same treatment as (2).
///
/// Returns a sorted, deduplicated `Vec<String>` of xnames.
pub async fn resolve_target_nodes(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hosts_expression: Option<&str>,
  hsm_group_name_arg_opt: Option<&str>,
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<Vec<String>, anyhow::Error> {
  if let Some(hosts_expr) = hosts_expression {
    resolve_hosts_expression(backend, shasta_token, hosts_expr, false).await
  } else if hsm_group_name_arg_opt.is_some()
    || settings_hsm_group_name_opt.is_some()
  {
    let hsm_group_name_vec =
      super::authorization::get_groups_names_available(
        backend,
        shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await
      .context("Failed to get available HSM group names")?;

    let hsm_members: Vec<String> = backend
      .get_member_vec_from_group_name_vec(shasta_token, &hsm_group_name_vec)
      .await
      .context("Could not fetch HSM group members")?;

    resolve_hosts_expression(
      backend,
      shasta_token,
      &hsm_members.join(","),
      false,
    )
    .await
  } else {
    anyhow::bail!(
      "No nodes provided. Please provide either a list of nodes \
       via --nodes or an HSM group via --hsm-group",
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ---- validate_xname_format ----

  #[test]
  fn valid_xname() {
    assert!(validate_xname_format("x1000c0s0b0n0"));
  }

  #[test]
  fn valid_xname_max_values() {
    assert!(validate_xname_format("x9999c7s64b1n7"));
  }

  #[test]
  fn invalid_xname_missing_prefix() {
    assert!(!validate_xname_format("1000c0s0b0n0"));
  }

  #[test]
  fn invalid_xname_bad_cabinet() {
    assert!(!validate_xname_format("x100c0s0b0n0"));
  }

  #[test]
  fn invalid_xname_slot_too_high() {
    assert!(!validate_xname_format("x1000c0s65b0n0"));
  }

  #[test]
  fn invalid_xname_board_too_high() {
    assert!(!validate_xname_format("x1000c0s0b2n0"));
  }

  #[test]
  fn invalid_xname_node_too_high() {
    assert!(!validate_xname_format("x1000c0s0b0n8"));
  }

  #[test]
  fn invalid_xname_chassis_too_high() {
    assert!(!validate_xname_format("x1000c8s0b0n0"));
  }

  #[test]
  fn invalid_xname_empty() {
    assert!(!validate_xname_format(""));
  }

  #[test]
  fn invalid_xname_garbage() {
    assert!(!validate_xname_format("not-an-xname"));
  }

  // ---- validate_nid_format ----

  #[test]
  fn valid_nid() {
    assert!(validate_nid_format("nid000001"));
  }

  #[test]
  fn valid_nid_all_zeros() {
    assert!(validate_nid_format("nid000000"));
  }

  #[test]
  fn invalid_nid_too_short() {
    assert!(!validate_nid_format("nid001"));
  }

  #[test]
  fn invalid_nid_too_long() {
    assert!(!validate_nid_format("nid0000001"));
  }

  #[test]
  fn invalid_nid_missing_prefix() {
    assert!(!validate_nid_format("000000001"));
  }

  #[test]
  fn invalid_nid_non_numeric() {
    assert!(!validate_nid_format("nid00000a"));
  }

  #[test]
  fn invalid_nid_uppercase() {
    // validate_nid_format lowercases for starts_with check
    // but strip_prefix("nid") on the original string fails
    // for uppercase input, so uppercase NIDs are rejected
    assert!(!validate_nid_format("NID000001"));
  }

  // ---- get_short_nid ----

  #[test]
  fn short_nid_valid() {
    assert_eq!(get_short_nid("nid000001").unwrap(), 1);
  }

  #[test]
  fn short_nid_larger_number() {
    assert_eq!(get_short_nid("nid001234").unwrap(), 1234);
  }

  #[test]
  fn short_nid_zero() {
    assert_eq!(get_short_nid("nid000000").unwrap(), 0);
  }

  #[test]
  fn short_nid_wrong_length() {
    assert!(get_short_nid("nid001").is_err());
  }

  #[test]
  fn short_nid_no_prefix() {
    assert!(get_short_nid("xxx000001").is_err());
  }

  // ---- string_vec_to_multi_line_string ----

  #[test]
  fn multi_line_none() {
    assert_eq!(string_vec_to_multi_line_string(None, 1), "");
  }

  #[test]
  fn multi_line_empty() {
    let nodes: Vec<String> = vec![];
    assert_eq!(string_vec_to_multi_line_string(Some(&nodes), 1), "");
  }

  #[test]
  fn multi_line_single_element() {
    let nodes = vec!["x1000c0s0b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 1),
      "x1000c0s0b0n0"
    );
  }

  #[test]
  fn multi_line_two_elements_one_column() {
    let nodes = vec!["x1000c0s0b0n0".to_string(), "x1000c0s1b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 1),
      "x1000c0s0b0n0,\nx1000c0s1b0n0"
    );
  }

  #[test]
  fn multi_line_two_elements_two_columns() {
    let nodes = vec!["x1000c0s0b0n0".to_string(), "x1000c0s1b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 2),
      "x1000c0s0b0n0,x1000c0s1b0n0"
    );
  }

  #[test]
  fn multi_line_three_elements_two_columns() {
    let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    // i=1: 1%2=1 -> comma, i=2: 2%2=0 -> newline
    assert_eq!(string_vec_to_multi_line_string(Some(&nodes), 2), "a,b,\nc");
  }

  // ---- helper ----

  /// Build a minimal `Component` with only `id` and `nid` populated;
  /// every other field is `None`.
  fn make_component(id: &str, nid: Option<usize>) -> Component {
    Component {
      id: Some(id.to_string()),
      r#type: None,
      state: None,
      flag: None,
      enabled: None,
      software_status: None,
      role: None,
      sub_role: None,
      nid,
      subtype: None,
      net_type: None,
      arch: None,
      class: None,
      reservation_disabled: None,
      locked: None,
    }
  }

  // ---- get_xname_from_nid_hostlist ----

  #[tokio::test]
  async fn nid_hostlist_matching_nids() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
      make_component("x1000c0s1b0n0", Some(2)),
      make_component("x1000c0s2b0n0", Some(3)),
    ];
    let nids = vec!["nid000001".to_string(), "nid000003".to_string()];

    let result = get_xname_from_nid_hostlist(&nids, &metadata).await.unwrap();
    assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s2b0n0"]);
  }

  #[tokio::test]
  async fn nid_hostlist_no_match() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
    ];
    let nids = vec!["nid000099".to_string()];

    let result = get_xname_from_nid_hostlist(&nids, &metadata).await.unwrap();
    assert!(result.is_empty());
  }

  #[tokio::test]
  async fn nid_hostlist_empty_inputs() {
    let result = get_xname_from_nid_hostlist(&[], &[]).await.unwrap();
    assert!(result.is_empty());
  }

  // ---- get_xname_from_xname_hostlist ----

  #[tokio::test]
  async fn xname_hostlist_matching_xnames() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
      make_component("x1000c0s1b0n0", Some(2)),
      make_component("x1000c0s2b0n0", Some(3)),
    ];
    let xnames = vec![
      "x1000c0s0b0n0".to_string(),
      "x1000c0s2b0n0".to_string(),
    ];

    let result = get_xname_from_xname_hostlist(&xnames, &metadata).await.unwrap();
    assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s2b0n0"]);
  }

  #[tokio::test]
  async fn xname_hostlist_no_match() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
    ];
    let xnames = vec!["x9999c0s0b0n0".to_string()];

    let result = get_xname_from_xname_hostlist(&xnames, &metadata).await.unwrap();
    assert!(result.is_empty());
  }

  #[tokio::test]
  async fn xname_hostlist_empty_inputs() {
    let result = get_xname_from_xname_hostlist(&[], &[]).await.unwrap();
    assert!(result.is_empty());
  }

  // ---- from_hosts_expression_to_xname_vec ----

  #[tokio::test]
  async fn hosts_expression_nid_list() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
      make_component("x1000c0s1b0n0", Some(2)),
      make_component("x1000c0s2b0n0", Some(3)),
    ];
    // Comma-separated NID hostlist
    let result = from_hosts_expression_to_xname_vec(
      "nid000001,nid000002",
      false,
      metadata,
    )
    .await
    .unwrap();
    assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s1b0n0"]);
  }

  #[tokio::test]
  async fn hosts_expression_xname_list() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
      make_component("x1000c0s1b0n0", Some(2)),
    ];
    let result = from_hosts_expression_to_xname_vec(
      "x1000c0s0b0n0,x1000c0s1b0n0",
      false,
      metadata,
    )
    .await
    .unwrap();
    assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s1b0n0"]);
  }

  #[tokio::test]
  async fn hosts_expression_invalid_input() {
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
    ];
    // "foobar" is neither a valid NID nor xname
    let result = from_hosts_expression_to_xname_vec(
      "foobar",
      false,
      metadata,
    )
    .await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn hosts_expression_nid_no_metadata_match_returns_error() {
    // All NIDs are valid but none match the metadata -> empty -> error
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(99)),
    ];
    let result = from_hosts_expression_to_xname_vec(
      "nid000001",
      false,
      metadata,
    )
    .await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn hosts_expression_include_siblings() {
    // Two nodes on the same blade (x1000c0s0b0), one on a different blade
    let metadata = vec![
      make_component("x1000c0s0b0n0", Some(1)),
      make_component("x1000c0s0b0n1", Some(2)), // sibling of n0
      make_component("x1000c0s1b0n0", Some(3)), // different blade
    ];
    // Request only nid000001 but include siblings
    let mut result = from_hosts_expression_to_xname_vec(
      "nid000001",
      true,
      metadata,
    )
    .await
    .unwrap();
    result.sort();
    assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s0b0n1"]);
  }
}
