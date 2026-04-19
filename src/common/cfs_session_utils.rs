use manta_backend_dispatcher::types::{
  cfs::session::CfsSessionGetResponse, Group,
};

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

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::{
    cfs::session::{Ansible, CfsSessionGetResponse, Target},
    Member,
  };

  /// Helper: create a Group (HSM group) with label and member xnames.
  fn make_hsm_group(label: &str, member_ids: Vec<&str>) -> Group {
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

  /// Helper: create a CFS session targeting HSM groups.
  fn make_session_with_groups(
    name: &str,
    groups: Vec<(&str, Vec<&str>)>,
  ) -> CfsSessionGetResponse {
    let cfs_groups = groups
      .into_iter()
      .map(|(n, members)| {
        manta_backend_dispatcher::types::cfs::session::Group {
          name: n.to_string(),
          members: members.into_iter().map(String::from).collect(),
        }
      })
      .collect();
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: None,
      ansible: None,
      target: Some(Target {
        definition: Some("dynamic".to_string()),
        groups: Some(cfs_groups),
        image_map: None,
      }),
      status: None,
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  /// Helper: create a CFS session targeting xnames via ansible limit.
  fn make_session_with_xnames(
    name: &str,
    xnames: &str,
  ) -> CfsSessionGetResponse {
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: None,
      ansible: Some(Ansible {
        config: None,
        limit: Some(xnames.to_string()),
        verbosity: None,
        passthrough: None,
      }),
      target: Some(Target {
        definition: Some("dynamic".to_string()),
        groups: None,
        image_map: None,
      }),
      status: None,
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  #[test]
  fn returns_true_when_session_targets_matching_hsm_group() {
    let session = make_session_with_groups(
      "sess1",
      vec![("compute", vec!["x1000c0s0b0n0"])],
    );
    let groups = vec![make_hsm_group("compute", vec!["x1000c0s0b0n0"])];
    assert!(check_cfs_session_against_groups_available(&session, groups));
  }

  #[test]
  fn returns_false_when_session_targets_different_hsm_group() {
    let session =
      make_session_with_groups("sess1", vec![("uan", vec!["x2000c0s0b0n0"])]);
    let groups = vec![make_hsm_group("compute", vec!["x1000c0s0b0n0"])];
    assert!(!check_cfs_session_against_groups_available(
      &session, groups
    ));
  }

  #[test]
  fn returns_true_when_session_xnames_all_in_group_members() {
    let session =
      make_session_with_xnames("sess1", "x1000c0s0b0n0,x1000c0s0b0n1");
    let groups = vec![make_hsm_group(
      "compute",
      vec!["x1000c0s0b0n0", "x1000c0s0b0n1", "x1000c0s0b0n2"],
    )];
    assert!(check_cfs_session_against_groups_available(&session, groups));
  }

  #[test]
  fn returns_false_when_session_xnames_not_all_in_any_group() {
    let session =
      make_session_with_xnames("sess1", "x1000c0s0b0n0,x9999c0s0b0n0");
    let groups = vec![make_hsm_group("compute", vec!["x1000c0s0b0n0"])];
    assert!(!check_cfs_session_against_groups_available(
      &session, groups
    ));
  }

  #[test]
  fn returns_false_when_no_groups_available() {
    let session = make_session_with_groups(
      "sess1",
      vec![("compute", vec!["x1000c0s0b0n0"])],
    );
    assert!(!check_cfs_session_against_groups_available(
      &session,
      vec![]
    ));
  }
}
