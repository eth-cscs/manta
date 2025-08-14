use std::collections::HashMap;

use comfy_table::{Cell, ContentArrangement, Table};
use csm_rs::node::types::NodeDetails;
use hostlist_parser::parse;
use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Component,
};
use regex::Regex;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

// Validate and get short nid
pub fn get_short_nid(long_nid: &str) -> Result<usize, Error> {
  // Validate nid has the right length
  if long_nid.len() != 9 {
    return Err(Error::Message(format!(
      "Nid '{}' not valid, Nid does not have 9 characters",
      long_nid
    )));
  }

  long_nid.strip_prefix("nid")
        .ok_or_else(|| Error::Message(format!("Nid '{}' not valid, 'nid' prefix missing", long_nid)))
        .and_then(|nid_number| nid_number.to_string().parse::<usize>()
                            .map_err(|e| Error::Message(format!("Intermediate operation to convert Nid {} from long to short format. Reason:\n{}", nid_number, e.to_string())))
        )
}

pub async fn get_xname_from_nid_hostlist(
  node_vec: &Vec<String>,
  node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
  // Convert long nids to short nids
  // Get xnames from short nids
  let short_nid_vec: Vec<usize> = node_vec
    .clone()
    .iter()
    .map(|nid_long| get_short_nid(nid_long))
    .collect::<Result<Vec<_>, Error>>()?;

  log::debug!("short Nid list expanded: {:?}", short_nid_vec);

  let xname_vec: Vec<String> = node_metadata_available_vec
    .into_iter()
    .filter(|node_metadata_available| {
      short_nid_vec.contains(&node_metadata_available.nid.unwrap())
    })
    .map(|node_metadata_available| node_metadata_available.id.as_ref().unwrap())
    .cloned()
    .collect();

  Ok(xname_vec)
}

pub async fn get_xname_from_xname_hostlist(
  node_vec: &Vec<String>,
  node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
  // If hostlist of XNAMEs, return hostlist expanded xnames
  // Validate XNAMEs
  log::debug!("XNAME format are valid");

  let xname_vec: Vec<String> = node_metadata_available_vec
    .into_iter()
    .filter(|node_metadata_available| {
      node_vec.contains(&node_metadata_available.id.as_ref().unwrap())
    })
    .map(|node_metadata_available| node_metadata_available.id.as_ref().unwrap())
    .cloned()
    .collect();

  Ok(xname_vec)
}

pub async fn get_xname_from_nid_regex(
  regex: &Regex,
  node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
  let xname_vec: Vec<String> = node_metadata_available_vec
    .clone()
    .into_iter()
    .filter(|node_metadata_available: &Component| {
      regex.is_match(&format!("nid{:06}", node_metadata_available.nid.unwrap()))
    })
    .map(|node_metadata_available| node_metadata_available.id.unwrap())
    .collect();

  Ok(xname_vec)
}

pub async fn get_xname_from_xname_regex(
  regex: &Regex,
  node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
  let xname_vec = node_metadata_available_vec
    .clone()
    .into_iter()
    .filter(|node_metadata_available: &Component| {
      regex.is_match(&node_metadata_available.id.as_ref().unwrap())
    })
    .map(|node_metadata_available| node_metadata_available.id.unwrap())
    .collect();

  Ok(xname_vec)
}

/// Translates and filters a 'host expression' into a list of xnames.
/// a host expression is a comma separated list of NIDs or XNAMEs, a regex or a hostlist
/// NOTE: regex expressions needs to be compared/filtered with a list of nodes available to the user
/// NOTE: user can provice a host expression and expand the list to all siblings
pub async fn from_hosts_expression_to_xname_vec(
  user_input: &str,
  is_include_siblings: bool,
  node_metadata_available_vec: Vec<Component>,
) -> Result<Vec<String>, Error> {
  // Check if hostlist
  // Expand user input to list of nids
  let hostlist_expanded_vec_rslt =
    parse(user_input).map_err(|e| Error::Message(e.to_string()));

  // Check if regex
  let regexexp_rslt =
    Regex::new(user_input).map_err(|e| Error::Message(e.to_string()));

  let xname_vec = if let Ok(node_vec) = hostlist_expanded_vec_rslt {
    log::debug!("Hostlist format is valid");
    // If hostlist, expand hostlist
    let xname_vec: Vec<String> = if validate_nid_format_vec(node_vec.clone()) {
      // If hostlist of NIDs, convert to xname
      // Validate NIDs
      log::debug!("NID format is valid");
      log::debug!("hostlist Nids: {}", user_input);
      log::debug!("hostlist Nids expanded: {:?}", node_vec);

      get_xname_from_nid_hostlist(&node_vec, &node_metadata_available_vec)
        .await?
    } else if validate_xname_format_vec(node_vec.clone()) {
      // If hostlist of XNAMEs, return hostlist expanded xnames
      // Validate XNAMEs
      log::debug!("NID format is valid");
      log::debug!("hostlist Nids: {}", user_input);
      log::debug!("hostlist Nids expanded: {:?}", node_vec);

      get_xname_from_xname_hostlist(&node_vec, &node_metadata_available_vec)
        .await?
    } else {
      return Err(Error::Message(format!(
                "Could not parse user input as a list of nodes from a hostlist expression.",
            )));
    };

    xname_vec
  } else if let Ok(regex) = regexexp_rslt {
    log::debug!("Regex format is valid");
    // If regex, return regex
    // Filter, validate and translate list of regex nids to xnames
    let xname_vec =
      get_xname_from_nid_regex(&regex, &node_metadata_available_vec).await?;

    log::debug!("Regex format: {}", regex);
    log::debug!("NID list from regex: {:?}", xname_vec);

    let xname_vec: Vec<String> = if xname_vec.is_empty() {
      log::debug!("No NIDs found from regex");
      // Filter, validate and translate list of regex xnames to xnames
      get_xname_from_xname_regex(&regex, &node_metadata_available_vec).await?
    } else {
      xname_vec
    };

    xname_vec
  } else {
    return Err(Error::Message(format!(
            "Could not parse user input as a list of nodes from a hostlist or regex expression.",
        )));
  };

  if xname_vec.is_empty() {
    return Err(Error::Message(format!(
            "Could not parse user input as a list of nodes from a hostlist or regex expression.",
        )));
  }

  // Include siblings if requested
  let xname_vec: Vec<String> = if is_include_siblings {
    log::debug!("Include siblings");
    let xname_blade_vec: Vec<String> = xname_vec
      .iter()
      .map(|xname| xname[0..10].to_string())
      .collect();

    log::debug!("XNAME blades:\n{:?}", xname_blade_vec);

    // Filter xnames to the ones the user has access to
    let xname_vec = node_metadata_available_vec
      .into_iter()
      .filter(|node_metadata_available| {
        xname_blade_vec.iter().any(|xname_blade| {
          node_metadata_available
            .id
            .as_ref()
            .unwrap()
            .starts_with(xname_blade)
        })
      })
      .map(|node_metadata_available| node_metadata_available.id.unwrap())
      .collect();

    xname_vec
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
) -> HashMap<String, Vec<String>> {
  // Create a summary of HSM groups and the list of members filtered by the list of nodes the
  // user is targeting
  let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

  /* // Get final list of xnames to operate on
  // Get list of HSM groups available
  // NOTE: HSM available are the ones the user has access to
  // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

  // Get all HSM groups in the system
  // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
  // information already filtered to the client:
  // hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
  // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
  // a Vec of HsmGroups the user has access to
  let hsm_group_vec_all =
      hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
          .await
          .expect("Error - fetching HSM groups"); */

  let hsm_name_available_vec =
    backend.get_group_name_available(auth_token).await.unwrap();

  // Get HSM group user has access to
  let hsm_group_available_map = backend
    .get_group_map_and_filter_by_group_vec(
      auth_token,
      hsm_name_available_vec
        .iter()
        .map(|hsm_name| hsm_name.as_str())
        .collect(),
    )
    .await
    .expect("ERROR - could not get HSM group summary");

  // Filter hsm group members
  for (hsm_name, hsm_members) in hsm_group_available_map {
    let xname_filtered: Vec<String> = hsm_members
      .iter()
      .filter(|&xname| xname_vec.contains(&xname))
      .cloned()
      .collect();
    if !xname_filtered.is_empty() {
      hsm_group_summary.insert(hsm_name, xname_filtered);
    }
  }

  hsm_group_summary
}

/// Check if input is a NID
pub fn validate_nid_format_vec(node_vec: Vec<String>) -> bool {
  node_vec.iter().all(|nid| validate_nid_format(nid))
}

/// Check if input is a NID
pub fn validate_nid_format(nid: &str) -> bool {
  nid.to_lowercase().starts_with("nid")
    && nid.len() == 9
    && nid
      .strip_prefix("nid")
      .is_some_and(|nid_number| nid_number.chars().all(char::is_numeric))
}

/// Validate xname is correct (it uses regex taken from HPE Cray CSM docs)
pub fn validate_xname_format_vec(node_vec: Vec<String>) -> bool {
  node_vec.iter().all(|nid| validate_xname_format(nid))
}

/// Validate xname is correct (it uses regex taken from HPE Cray CSM docs)
pub fn validate_xname_format(xname: &str) -> bool {
  let xname_re =
    Regex::new(r"^x\d{4}c[0-7]s([0-9]|[1-5][0-9]|6[0-4])b[0-1]n[0-7]$")
      .unwrap();

  xname_re.is_match(xname)
}

pub fn print_table(nodes_status: Vec<NodeDetails>) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "XNAME",
    "NID",
    "HSM",
    "Power",
    "Runtime Config",
    "Config Stat",
    "Enabled",
    "# Error",
    "Image ID",
  ]);

  for node_status in nodes_status {
    let mut node_vec: Vec<String> = node_status
      .hsm
      .split(",")
      .map(|xname_str| xname_str.trim().to_string())
      .collect();
    node_vec.sort();

    table.add_row(vec![
      Cell::new(node_status.xname),
      Cell::new(node_status.nid),
      Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
      Cell::new(node_status.power_status),
      Cell::new(node_status.desired_configuration),
      Cell::new(node_status.configuration_status),
      Cell::new(node_status.enabled),
      Cell::new(node_status.error_count),
      Cell::new(node_status.boot_image_id),
    ]);
  }

  println!("{table}");
}

pub fn print_table_wide(nodes_status: Vec<NodeDetails>) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "XNAME",
    "NID",
    "HSM",
    "Power",
    "Runtime Config",
    "Config Status",
    "Enabled",
    "Error #",
    "Image ID",
    "Kernel Params",
  ]);

  for node_status in nodes_status {
    let kernel_params_vec: Vec<&str> =
      node_status.kernel_params.split_whitespace().collect();
    let cell_max_width = kernel_params_vec
      .iter()
      .map(|value| value.len())
      .max()
      .unwrap_or(0);

    let mut kernel_params_string: String = kernel_params_vec[0].to_string();
    let mut cell_width = kernel_params_string.len();

    for kernel_param in kernel_params_vec.iter().skip(1) {
      cell_width += kernel_param.len();

      if cell_width + kernel_param.len() >= cell_max_width {
        kernel_params_string.push_str("\n");
        cell_width = 0;
      } else {
        kernel_params_string.push_str(" ");
      }

      kernel_params_string.push_str(kernel_param);
    }

    let mut node_vec: Vec<String> = node_status
      .hsm
      .split(",")
      .map(|xname_str| xname_str.trim().to_string())
      .collect();
    node_vec.sort();

    table.add_row(vec![
      Cell::new(node_status.xname),
      Cell::new(node_status.nid),
      Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
      Cell::new(node_status.power_status),
      Cell::new(node_status.desired_configuration),
      Cell::new(node_status.configuration_status),
      Cell::new(node_status.enabled),
      Cell::new(node_status.error_count),
      Cell::new(node_status.boot_image_id),
      Cell::new(kernel_params_string),
    ]);
  }

  println!("{table}");
}

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

  let mut table = Table::new();

  table.set_header(vec!["Boot configuration name", "Num nodes"]);

  for (config_name, counter) in boot_configuration_counters {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(config_name),
        Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }

  println!("{table}");

  let mut table = Table::new();

  table.set_header(vec!["Boot image id", "Num nodes"]);

  for (image_id, counter) in boot_image_counters {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(image_id),
        Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }

  println!("{table}");

  let mut table = Table::new();

  table.set_header(vec!["Runtime configuration name", "Num nodes"]);

  for (config_name, counter) in runtime_configuration_counters {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(config_name),
        Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }

  println!("{table}");
}

pub fn nodes_to_string_format_discrete_columns(
  nodes: Option<&Vec<String>>,
  num_columns: usize,
) -> String {
  let mut members: String;

  match nodes {
    Some(nodes) if !nodes.is_empty() => {
      members = nodes[0].clone(); // take first element

      for (i, _) in nodes.iter().enumerate().skip(1) {
        // iterate for the rest of the list
        if i % num_columns == 0 {
          // breaking the cell content into multiple lines (only 2 xnames per line)

          members.push_str(",\n");
        } else {
          members.push(',');
        }

        members.push_str(&nodes[i]);
      }
    }
    _ => members = "".to_string(),
  }

  members
}

pub fn string_vec_to_multi_line_string(
  nodes: Option<&Vec<String>>,
  num_columns: usize,
) -> String {
  let mut members: String;

  match nodes {
    Some(nodes) if !nodes.is_empty() => {
      members = nodes.first().unwrap().to_string(); // take first element

      for (i, _) in nodes.iter().enumerate().skip(1) {
        // iterate for the rest of the list
        if i % num_columns == 0 {
          // breaking the cell content into multiple lines (only 2 xnames per line)

          members.push_str(",\n");
        } else {
          members.push(',');
        }

        members.push_str(&nodes[i]);
      }
    }
    _ => members = "".to_string(),
  }

  members
}
