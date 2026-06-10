//! Node-expression resolution: parsing hostlist strings, NID-to-xname
//! translation, HSM-group expansion, and the authorization helpers
//! that validate the caller can act on the resolved set.

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

use crate::server::common::app_context::InfraContext;

/// Length of a NID string, e.g. "nid000001" = 9 characters.
const NID_STRING_LENGTH: usize = 9;

/// Length of the xname blade prefix, e.g. "x1000c7s0b" = 10 characters.
const XNAME_BLADE_PREFIX_LEN: usize = 10;

// Validate and get short nid
fn get_short_nid(long_nid: &str) -> Result<usize, Error> {
  if long_nid.len() != NID_STRING_LENGTH {
    return Err(Error::InvalidNodeId(format!(
      "Nid '{long_nid}' not valid, Nid does not have {NID_STRING_LENGTH} characters"
    )));
  }

  let nid_number = long_nid.strip_prefix("nid").ok_or_else(|| {
    Error::InvalidNodeId(format!(
      "Nid '{long_nid}' not valid, 'nid' prefix missing"
    ))
  })?;

  nid_number.parse::<usize>().map_err(|e| {
    Error::InvalidNodeId(format!(
      "Could not convert Nid '{nid_number}' from long to short format: {e}"
    ))
  })
}

/// Resolve a NID hostlist expression to xnames by
/// cross-referencing available node metadata.
pub fn get_xname_from_nid_hostlist(
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

  // Build a HashSet once so the per-component lookup below is O(1).
  // The previous `short_nid_vec.contains(&nid)` was O(N) — at cluster
  // scale (say a hostlist `nid[1-5000]` against ~5k components) that
  // turned into a 25M-comparison filter on every resolve.
  let short_nid_set: std::collections::HashSet<usize> =
    short_nid_vec.iter().copied().collect();
  let xname_vec: Vec<String> = node_metadata_available_vec
    .iter()
    .filter(|node_metadata_available| {
      node_metadata_available
        .nid
        .is_some_and(|nid| short_nid_set.contains(&nid))
    })
    .filter_map(|node_metadata_available| {
      node_metadata_available.id.as_ref().cloned()
    })
    .collect();

  Ok(xname_vec)
}

/// Filter available node metadata to only those xnames
/// present in `node_vec`.
pub fn get_xname_from_xname_hostlist(
  node_vec: &[String],
  node_metadata_available_vec: &[Component],
) -> Result<Vec<String>, Error> {
  // If hostlist of XNAMEs, return hostlist expanded xnames
  // Validate XNAMEs.
  //
  // Hash the requested-xname list once — same reasoning as
  // `get_xname_from_nid_hostlist`: at cluster scale the
  // `node_vec.contains(id)` filter was O(N·M).
  let node_set: std::collections::HashSet<&str> =
    node_vec.iter().map(String::as_str).collect();
  let xname_vec: Vec<String> = node_metadata_available_vec
    .iter()
    .filter(|node_metadata_available| {
      node_metadata_available
        .id
        .as_ref()
        .is_some_and(|id| node_set.contains(id.as_str()))
    })
    .filter_map(|node_metadata_available| {
      node_metadata_available.id.as_ref().cloned()
    })
    .collect();

  Ok(xname_vec)
}

/// Convenience wrapper that fetches node metadata from the backend
/// and resolves a hosts expression to a sorted, deduplicated list
/// of xnames.
///
/// Combines the two-step pattern of
/// [`InfraContext::get_node_metadata_available`] followed by
/// [`from_hosts_expression_to_xname_vec`] that recurs in many
/// command files.
pub async fn from_user_hosts_expression_to_xname_vec(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  hosts_expression: &str,
  is_include_siblings: bool,
) -> Result<Vec<String>, Error> {
  let node_metadata_available_vec =
    infra.backend.get_node_metadata_available(shasta_token).await?;

  let mut xname_vec = from_hosts_expression_to_xname_vec(
    hosts_expression,
    is_include_siblings,
    &node_metadata_available_vec,
  )?;

  xname_vec.sort();
  xname_vec.dedup();

  Ok(xname_vec)
}

/// Translates a 'host expression' into a list of xnames.
///
/// A host expression is a comma-separated list of NIDs or xnames, a regex,
/// or a hostlist. When `is_include_siblings` is true, the resulting xnames
/// are expanded to include all siblings (other nodes on the same BMC).
pub fn from_hosts_expression_to_xname_vec(
  user_input: &str,
  is_include_siblings: bool,
  node_metadata_available_vec: &[Component],
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

        get_xname_from_nid_hostlist(&node_vec, node_metadata_available_vec)?
      } else if validate_xname_format_vec(&node_vec) {
        tracing::debug!("XNAME format is valid");
        tracing::debug!("hostlist XNAMEs: {}", user_input);
        tracing::debug!("hostlist XNAMEs expanded: {:?}", node_vec);

        get_xname_from_xname_hostlist(&node_vec, node_metadata_available_vec)?
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

    // Include siblings: keep any node whose xname shares a blade
    // prefix with one of the resolved xnames.
    node_metadata_available_vec
      .iter()
      .filter(|node_metadata_available| {
        node_metadata_available.id.as_ref().is_some_and(|id| {
          xname_blade_vec
            .iter()
            .any(|xname_blade| id.starts_with(xname_blade))
        })
      })
      .filter_map(|node_metadata_available| node_metadata_available.id.as_ref())
      .cloned()
      .collect()
  } else {
    xname_vec
  };

  Ok(xname_vec)
}

/// Group the supplied xnames by their parent HSM group.
///
/// Fetches the HSM groups the caller can access, then for each group
/// returns the intersection of its membership with `xname_vec`.
/// Groups whose intersection is empty are omitted, so the returned
/// map contains only groups that actually contribute at least one
/// matching node.
pub async fn get_curated_group_from_xname_hostlist(
  infra: &InfraContext<'_>,
  auth_token: &str,
  xname_vec: &[String],
) -> Result<HashMap<String, Vec<String>>, Error> {
  let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

  let hsm_name_available_vec =
    infra.backend.get_group_name_available(auth_token).await?;

  let names_ref: Vec<&str> =
    hsm_name_available_vec.iter().map(String::as_str).collect();
  let hsm_group_available_map = infra
    .backend
    .get_group_map_and_filter_by_group_vec(auth_token, &names_ref)
    .await?;

  // Filter hsm group members. Pre-compute a hash of the requested
  // xname set once — the outer loop is over groups and the inner
  // `xname_vec.contains(xname)` would otherwise re-scan the full
  // requested list per member per group (groups × members × xnames).
  let xname_set: std::collections::HashSet<&str> =
    xname_vec.iter().map(String::as_str).collect();
  for (hsm_name, hsm_members) in hsm_group_available_map {
    let xname_filtered: Vec<String> = hsm_members
      .iter()
      .filter(|xname| xname_set.contains(xname.as_str()))
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
pub(crate) fn validate_xname_format(xname: &str) -> bool {
  XNAME_RE.is_match(xname)
}

/// Resolve target nodes from either a hosts expression, an
/// explicit HSM group name, or the settings-level HSM group.
///
/// Priority order:
/// 1. `hosts_expression` — parsed and validated via
///    [`from_user_hosts_expression_to_xname_vec`].
/// 2. `group_name_arg_opt` — the group name supplied by the CLI's
///    `--group` flag (also accepted as `--hsm-group`); validated for
///    access via
///    [`crate::service::authorization::validate_user_group_access`],
///    then expanded to member xnames.
/// 3. `settings_group_name_opt` — the group configured in
///    `cli.toml`'s `parent_hsm_group`; same treatment as (2).
///
/// Returns a sorted, deduplicated `Vec<String>` of xnames.
pub async fn resolve_target_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  hosts_expression_opt: Option<&str>,
  group_name_arg_opt: Option<&str>,
  settings_group_name_opt: Option<&str>,
) -> Result<Vec<String>, Error> {
  if let Some(hosts_expr) = hosts_expression_opt {
    from_user_hosts_expression_to_xname_vec(infra, token, hosts_expr, false)
      .await
  } else if let Some(target_group) =
    group_name_arg_opt.or(settings_group_name_opt)
  {
    crate::service::authorization::validate_user_group_access(
      infra,
      token,
      target_group,
    )
    .await?;

    infra
      .backend
      .get_member_vec_from_group_name_vec(token, &[target_group.to_string()])
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
