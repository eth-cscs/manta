use std::collections::HashMap;

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::{
  cli::commands::hw_cluster_common::utils::{
    calculate_hsm_hw_component_summary, fetch_hsm_hw_inventory,
    print_hsm_group_json, resolve_hw_description_to_xnames,
  },
  common::app_context::AppContext,
};

/// Determines whether the hw cluster operation moves nodes
/// into the target (Pin) or releases them back (Unpin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwClusterMode {
  Pin,
  Unpin,
}

/// Result of an `apply hw-configuration` operation.
pub struct ApplyHwResult {
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Core logic for hardware cluster pin/unpin — no terminal interaction.
#[allow(clippy::too_many_arguments)]
pub async fn exec_with_backend(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  mode: HwClusterMode,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
  delete_empty_parent_hsm_group: bool,
) -> Result<ApplyHwResult, Error> {
  let (user_defined_hw_component_vec, user_defined_hw_component_count_hashmap) =
    parse_hw_pattern_usize(target_hsm_group_name, pattern)?;

  let mem_lcm = super::MEMORY_CAPACITY_LCM;

  ensure_target_group_exists(
    backend,
    shasta_token,
    target_hsm_group_name,
    dryrun,
    create_target_hsm_group,
  )
  .await?;

  let (
    target_hsm_group_member_vec,
    target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_hw_component_vec,
    target_hsm_group_name,
    mem_lcm,
  )
  .await?;

  tracing::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary
  );

  let (
    parent_hsm_group_member_vec,
    parent_hsm_node_hw_component_count_vec,
    _parent_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_hw_component_vec,
    parent_hsm_group_name,
    mem_lcm,
  )
  .await?;

  validate_resource_sufficiency(
    &target_hsm_node_hw_component_count_vec,
    &parent_hsm_node_hw_component_count_vec,
    &user_defined_hw_component_count_hashmap,
  )?;

  let (
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
  ) = resolve_hw_description_to_xnames(
    mode,
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
    user_defined_hw_component_count_hashmap,
  )
  .await?;

  let target_hsm_node_vec: Vec<String> = target_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  let parent_hsm_node_vec: Vec<String> = parent_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  apply_group_updates(
    backend,
    shasta_token,
    target_hsm_group_name,
    parent_hsm_group_name,
    &target_hsm_group_member_vec,
    &parent_hsm_group_member_vec,
    &target_hsm_node_vec,
    &parent_hsm_node_vec,
    dryrun,
    delete_empty_parent_hsm_group,
  )
  .await?;

  Ok(ApplyHwResult {
    target_nodes: target_hsm_node_vec,
    parent_nodes: parent_hsm_node_vec,
  })
}

/// Execute a hardware cluster pin or unpin operation,
/// moving nodes between target and parent HSM groups.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  mode: HwClusterMode,
  ctx: &AppContext<'_>,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
  delete_empty_parent_hsm_group: bool,
) -> Result<(), Error> {
  let result = exec_with_backend(
    ctx.infra.backend,
    mode,
    shasta_token,
    target_hsm_group_name,
    parent_hsm_group_name,
    pattern,
    dryrun,
    create_target_hsm_group,
    delete_empty_parent_hsm_group,
  )
  .await?;

  print_hsm_group_json(target_hsm_group_name, &result.target_nodes)?;
  print_hsm_group_json(parent_hsm_group_name, &result.parent_nodes)?;

  Ok(())
}

/// Parse user pattern `"a100:4:epyc:10"` into hw component
/// names and a hashmap of `{component -> count}` as `usize`.
fn parse_hw_pattern_usize(
  target_hsm_group_name: &str,
  pattern: &str,
) -> Result<(Vec<String>, HashMap<String, usize>), Error> {
  let pattern = format!("{}:{}", target_hsm_group_name, pattern);
  tracing::info!("pattern: {}", pattern);

  let pattern_lowercase = pattern.to_lowercase();

  let (_group_name, pattern_hw_component) =
    pattern_lowercase.split_once(':').context(
      "Invalid pattern format: \
       expected 'group:component:count'",
    )?;

  let pattern_element_vec: Vec<&str> =
    pattern_hw_component.split(':').collect();

  if !pattern_element_vec.len().is_multiple_of(2) {
    bail!(
      "Error in pattern: odd number of elements. \
       Expected pairs of <hw component>:<count>. \
       eg a100:4:epyc:10:instinct:8",
    );
  }

  let mut hw_component_count: HashMap<String, usize> = HashMap::new();

  for chunk in pattern_element_vec.chunks_exact(2) {
    if let Ok(count) = chunk[1].parse::<usize>() {
      hw_component_count.insert(chunk[0].to_string(), count);
    } else {
      bail!(
        "Error in pattern. Please make sure to follow \
         <hsm name>:<hw component>:<counter>:... \
         eg <tasna>:a100:4:epyc:10:instinct:8",
      );
    }
  }

  tracing::info!(
    "User defined hw components with counters: {:?}",
    hw_component_count
  );

  let mut hw_component_vec: Vec<String> =
    hw_component_count.keys().cloned().collect();
  hw_component_vec.sort();

  Ok((hw_component_vec, hw_component_count))
}

/// Ensure the target HSM group exists, creating it if
/// `create_target_hsm_group` is set.
async fn ensure_target_group_exists(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
) -> Result<(), Error> {
  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      tracing::debug!("Target HSM group '{}' exists, good.", target_hsm_group_name);
      Ok(())
    }
    Err(_) => {
      if !create_target_hsm_group {
        bail!(
          "Target HSM group '{}' does not exist, \
           but the option to create the group was \
           NOT specified, cannot continue.",
          target_hsm_group_name,
        );
      }
      tracing::info!(
        "Target HSM group '{}' does not exist, \
         but the option to create the group has \
         been selected, creating it now.",
        target_hsm_group_name
      );
      if dryrun {
        bail!(
          "Dryrun selected, cannot create the \
           new group and continue.",
        );
      }
      let group = Group {
        label: target_hsm_group_name.to_string(),
        description: None,
        tags: None,
        members: None,
        exclusive_group: Some("false".to_string()),
      };
      let _ = backend
        .add_group(shasta_token, group)
        .await
        .context("Unable to create new target HSM group")?;
      Ok(())
    }
  }
}

/// Validate that combined target+parent resources can
/// fulfill the user request.
fn validate_resource_sufficiency(
  target_hw: &[(String, HashMap<String, usize>)],
  parent_hw: &[(String, HashMap<String, usize>)],
  requested: &HashMap<String, usize>,
) -> Result<(), Error> {
  let mut combined = parent_hw.to_vec();
  for elem in target_hw {
    if !parent_hw.iter().any(|(xname, _)| xname.eq(&elem.0)) {
      combined.push(elem.clone());
    }
  }

  let combined_summary = calculate_hsm_hw_component_summary(&combined);

  for (hw_component, qty) in requested {
    if combined_summary
      .get(hw_component)
      .is_none_or(|value| value < qty)
    {
      bail!(
        "There are not enough resources \
         to fulfill user request.",
      );
    }
  }

  Ok(())
}

/// Apply group membership updates to both target and parent
/// HSM groups. Optionally deletes the parent group if it
/// becomes empty.
#[allow(clippy::too_many_arguments)]
async fn apply_group_updates(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  shasta_token: &str,
  target_group: &str,
  parent_group: &str,
  old_target_members: &[String],
  old_parent_members: &[String],
  new_target_members: &[String],
  new_parent_members: &[String],
  dryrun: bool,
  delete_empty_parent: bool,
) -> Result<(), Error> {
  // Update target group
  tracing::info!("Updating target HSM group '{}' members", target_group);
  if dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    backend
      .update_group_members(
        shasta_token,
        target_group,
        &old_target_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &new_target_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await
      .context("Failed to update target HSM group members")?;
  }

  // Update parent group
  tracing::info!("Updating parent HSM group '{}' members", parent_group);
  if dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    let parent_will_be_empty =
      old_target_members.len() == old_parent_members.len();
    backend
      .update_group_members(
        shasta_token,
        parent_group,
        &old_parent_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &new_parent_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await
      .context("Failed to update parent HSM group members")?;

    if parent_will_be_empty && delete_empty_parent {
      tracing::info!(
        "Parent HSM group '{}' is now empty and \
         the option to delete empty groups has \
         been selected, removing it.",
        parent_group
      );
      match backend.delete_group(shasta_token, parent_group).await {
        Ok(_) => {
          tracing::info!("HSM group removed successfully.")
        }
        Err(e) => tracing::debug!(
          "Error removing the HSM group. \
           This always fails, ignore please. \
           Reported: {}",
          e
        ),
      };
    } else if parent_will_be_empty {
      tracing::debug!(
        "Parent HSM group '{}' is now empty and \
         the option to delete empty groups has \
         NOT been selected, will not remove it.",
        parent_group
      )
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  // ---- parse_hw_pattern_usize ----

  #[test]
  fn parse_hw_pattern_usize_valid() {
    let (names, counts) =
      parse_hw_pattern_usize("tasna", "a100:4:epyc:10").unwrap();
    assert_eq!(names, vec!["a100", "epyc"]);
    assert_eq!(counts.get("a100"), Some(&4));
    assert_eq!(counts.get("epyc"), Some(&10));
  }

  #[test]
  fn parse_hw_pattern_usize_single_pair() {
    let (names, counts) =
      parse_hw_pattern_usize("group1", "instinct:8").unwrap();
    assert_eq!(names, vec!["instinct"]);
    assert_eq!(counts.get("instinct"), Some(&8));
  }

  #[test]
  fn parse_hw_pattern_usize_odd_elements_errors() {
    assert!(parse_hw_pattern_usize("g", "a100:4:epyc").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_non_numeric_count_errors() {
    assert!(parse_hw_pattern_usize("g", "a100:four").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_negative_count_errors() {
    // usize cannot be negative
    assert!(parse_hw_pattern_usize("g", "a100:-3").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_sorted_output() {
    let (names, _) =
      parse_hw_pattern_usize("g", "zebra:1:alpha:2:mid:3").unwrap();
    assert_eq!(names, vec!["alpha", "mid", "zebra"]);
  }

  #[test]
  fn parse_hw_pattern_usize_lowercased() {
    // Pattern should be lowercased
    let (names, counts) = parse_hw_pattern_usize("GROUP", "A100:4").unwrap();
    assert_eq!(names, vec!["a100"]);
    assert_eq!(counts.get("a100"), Some(&4));
  }

  // ---- validate_resource_sufficiency ----

  #[test]
  fn validate_sufficiency_passes() {
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let parent_hw = vec![(
      "x1000c0s1b0n0".to_string(),
      HashMap::from([("a100".to_string(), 8)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 10)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_ok()
    );
  }

  #[test]
  fn validate_sufficiency_fails_insufficient() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 2)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 10)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_err()
    );
  }

  #[test]
  fn validate_sufficiency_fails_missing_component() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 10)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 1)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_err()
    );
  }

  #[test]
  fn validate_sufficiency_exact_match() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 4)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_ok()
    );
  }

  #[test]
  fn validate_sufficiency_combines_target_and_parent() {
    // Target has a node not in parent — should be combined
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 3)]),
    )];
    let parent_hw = vec![(
      "x1000c0s1b0n0".to_string(),
      HashMap::from([("a100".to_string(), 3)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 6)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_ok()
    );
  }

  #[test]
  fn validate_sufficiency_no_double_count_overlap() {
    // Target node IS in parent — should NOT be double-counted
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    // Total available is 4, not 8
    let requested = HashMap::from([("a100".to_string(), 5)]);
    assert!(
      validate_resource_sufficiency(&target_hw, &parent_hw, &requested,)
        .is_err()
    );
  }
}
