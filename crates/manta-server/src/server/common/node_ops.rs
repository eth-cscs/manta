use std::collections::HashMap;
use std::sync::LazyLock;

use hostlist_parser::parse;
use manta_backend_dispatcher::{
  error::Error,
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
fn get_short_nid(long_nid: &str) -> Result<usize, Error> {
  if long_nid.len() != NID_STRING_LENGTH {
    return Err(Error::InvalidNodeId(format!(
      "Nid '{}' not valid, Nid does not have {} characters",
      long_nid, NID_STRING_LENGTH
    )));
  }

  let nid_number = long_nid.strip_prefix("nid").ok_or_else(|| {
    Error::InvalidNodeId(format!(
      "Nid '{}' not valid, 'nid' prefix missing",
      long_nid
    ))
  })?;

  nid_number.parse::<usize>().map_err(|e| {
    Error::InvalidNodeId(format!(
      "Could not convert Nid '{}' from long to short format: {}",
      nid_number, e
    ))
  })
}

/// Resolve a NID hostlist expression to xnames by
/// cross-referencing available node metadata.
pub async fn get_xname_from_nid_hostlist(
  node_vec: &[String],
  node_metadata_available_vec: &[Component],
) -> Result<Vec<String>, Error> {
  // Convert long nids to short nids
  // Get xnames from short nids
  let short_nid_vec: Vec<usize> = node_vec
    .iter()
    .map(|nid_long| get_short_nid(nid_long))
    .collect::<Result<Vec<_>, _>>()?;

  tracing::debug!("short Nid list expanded: {:?}", short_nid_vec);

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
) -> Result<Vec<String>, Error> {
  // If hostlist of XNAMEs, return hostlist expanded xnames
  // Validate XNAMEs
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
) -> Result<Vec<String>, Error> {
  let node_metadata_available_vec =
    backend.get_node_metadata_available(shasta_token).await?;

  let mut xname_vec = from_hosts_expression_to_xname_vec(
    hosts_expression,
    is_include_siblings,
    node_metadata_available_vec,
  )
  .await?;

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
) -> Result<Vec<String>, Error> {
  let hostlist_expanded_vec_rslt =
    parse(user_input).map_err(|e| Error::InvalidNodeId(e.to_string()));

  let xname_vec = match hostlist_expanded_vec_rslt {
    Ok(node_vec) => {
      tracing::debug!("Hostlist format is valid");
      let xname_vec: Vec<String> = if validate_nid_format_vec(&node_vec) {
        tracing::debug!("NID format is valid");
        tracing::debug!("hostlist Nids: {}", user_input);
        tracing::debug!("hostlist Nids expanded: {:?}", node_vec);

        get_xname_from_nid_hostlist(&node_vec, &node_metadata_available_vec)
          .await?
      } else if validate_xname_format_vec(&node_vec) {
        tracing::debug!("XNAME format is valid");
        tracing::debug!("hostlist XNAMEs: {}", user_input);
        tracing::debug!("hostlist XNAMEs expanded: {:?}", node_vec);

        get_xname_from_xname_hostlist(&node_vec, &node_metadata_available_vec)
          .await?
      } else {
        return Err(Error::BadRequest(
          "Could not parse user input as a list of nodes from a hostlist expression."
            .to_string(),
        ));
      };

      xname_vec
    }
    Err(e) => {
      return Err(Error::BadRequest(format!(
        "Could not parse user input as a list of nodes from a hostlist or regex expression: {e}"
      )));
    }
  };

  if xname_vec.is_empty() {
    return Err(Error::BadRequest(
      "Could not parse user input as a list of nodes from a hostlist or regex expression."
        .to_string(),
    ));
  }

  // Include siblings if requested
  let xname_vec: Vec<String> = if is_include_siblings {
    tracing::debug!("Include siblings");
    let xname_blade_vec: Vec<String> = xname_vec
      .iter()
      .map(|xname| {
        xname
          .get(0..XNAME_BLADE_PREFIX_LEN)
          .unwrap_or(xname)
          .to_string()
      })
      .collect();

    tracing::debug!("XNAME blades:\n{:?}", xname_blade_vec);

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
) -> Result<HashMap<String, Vec<String>>, Error> {
  let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

  let hsm_name_available_vec =
    backend.get_group_name_available(auth_token).await?;

  let hsm_group_available_map = backend
    .get_group_map_and_filter_by_group_vec(
      auth_token,
      &hsm_name_available_vec
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>(),
    )
    .await?;

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

fn validate_nid_format_vec(node_vec: &[String]) -> bool {
  node_vec.iter().all(|nid| validate_nid_format(nid))
}

fn validate_nid_format(nid: &str) -> bool {
  nid.to_lowercase().starts_with("nid")
    && nid.len() == 9
    && nid
      .strip_prefix("nid")
      .is_some_and(|nid_number| nid_number.chars().all(char::is_numeric))
}

fn validate_xname_format_vec(node_vec: &[String]) -> bool {
  node_vec.iter().all(|nid| validate_xname_format(nid))
}

/// Return `true` if `xname` matches the HPE Cray xname regex.
pub fn validate_xname_format(xname: &str) -> bool {
  XNAME_RE.is_match(xname)
}

// `string_vec_to_multi_line_string` (display-only helper) moved to
// `crate::cli::common::display::string_vec_to_multi_line_string`.
// The server never used it; its tests moved alongside it.

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
) -> Result<Vec<String>, Error> {
  if let Some(hosts_expr) = hosts_expression {
    resolve_hosts_expression(backend, shasta_token, hosts_expr, false).await
  } else if hsm_group_name_arg_opt.is_some()
    || settings_hsm_group_name_opt.is_some()
  {
    let hsm_group_name_vec =
      crate::server::common::authorization::get_groups_names_available(
        backend,
        shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await?;

    let hsm_members: Vec<String> = backend
      .get_member_vec_from_group_name_vec(shasta_token, &hsm_group_name_vec)
      .await?;

    resolve_hosts_expression(
      backend,
      shasta_token,
      &hsm_members.join(","),
      false,
    )
    .await
  } else {
    Err(Error::BadRequest(
      "No nodes provided. Please provide either a list of nodes \
       via --nodes or an HSM group via --hsm-group"
        .to_string(),
    ))
  }
}

#[cfg(test)]
mod tests;
