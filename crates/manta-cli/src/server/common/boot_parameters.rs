use manta_backend_dispatcher::types::{bss::BootParameters, Group};

/// Get a vector of boot parameters that are restricted based on the groups available to the user.
pub fn get_restricted_boot_parameters(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<BootParameters> {
  let group_members: Vec<String> = group_available_vec
    .iter()
    .flat_map(|group| group.get_members())
    .collect();

  boot_parameter_vec
    .iter()
    .filter(|boot_param| {
      group_members
        .iter()
        .any(|gma| boot_param.hosts.contains(gma))
    })
    .cloned()
    .collect::<Vec<BootParameters>>()
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::Member;

  /// Helper: create a Group with given label and member xnames.
  fn make_group(label: &str, member_ids: Vec<&str>) -> Group {
    Group {
      label: label.to_string(),
      description: None,
      tags: None,
      members: Some(Member {
        ids: Some(member_ids.into_iter().map(String::from).collect()),
      }),
      exclusive_group: None,
    }
  }

  /// Helper: create a BootParameters with given hosts.
  fn make_boot_params(hosts: Vec<&str>) -> BootParameters {
    BootParameters {
      hosts: hosts.into_iter().map(String::from).collect(),
      ..Default::default()
    }
  }

  #[test]
  fn filters_boot_params_by_group_membership() {
    let groups =
      vec![make_group("grp1", vec!["x1000c0s0b0n0", "x1000c0s0b0n1"])];
    let boot_params = vec![
      make_boot_params(vec!["x1000c0s0b0n0"]),
      make_boot_params(vec!["x9999c0s0b0n0"]),
      make_boot_params(vec!["x1000c0s0b0n1"]),
    ];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].hosts, vec!["x1000c0s0b0n0"]);
    assert_eq!(result[1].hosts, vec!["x1000c0s0b0n1"]);
  }

  #[test]
  fn returns_empty_when_no_group_members_match() {
    let groups = vec![make_group("grp1", vec!["x1000c0s0b0n0"])];
    let boot_params = vec![make_boot_params(vec!["x9999c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert!(result.is_empty());
  }

  #[test]
  fn returns_empty_when_groups_are_empty() {
    let boot_params = vec![make_boot_params(vec!["x1000c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&[], &boot_params);
    assert!(result.is_empty());
  }

  #[test]
  fn returns_empty_when_boot_params_are_empty() {
    let groups = vec![make_group("grp1", vec!["x1000c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&groups, &[]);
    assert!(result.is_empty());
  }

  #[test]
  fn aggregates_members_across_multiple_groups() {
    let groups = vec![
      make_group("grp1", vec!["x1000c0s0b0n0"]),
      make_group("grp2", vec!["x2000c0s0b0n0"]),
    ];
    let boot_params = vec![
      make_boot_params(vec!["x1000c0s0b0n0"]),
      make_boot_params(vec!["x2000c0s0b0n0"]),
      make_boot_params(vec!["x3000c0s0b0n0"]),
    ];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert_eq!(result.len(), 2);
  }
}
