//! Offline test driving [`Index::from_snapshots`] with the checked-in
//! prealps capture (`testdata/groups-prealps.json`, the verbatim
//! payload of `GET /api/v1/groups` against the CSCS prealps test site).
//!
//! The assertions mirror the fixture's "notable properties" documented
//! in ROADMAP.md: the site-umbrella group, empty groups, and a node
//! with overlapping group membership.

use std::collections::BTreeMap;

use manta_cache::{Index, NodeMembership, SiteSnapshot};
use serde::Deserialize;

/// Subset of the `GET /api/v1/groups` group object the test consumes;
/// serde ignores `description`, `exclusiveGroup`, and `tags`.
#[derive(Deserialize)]
struct Group {
  label: String,
  members: Members,
}

#[derive(Deserialize)]
struct Members {
  ids: Vec<String>,
}

/// Parse the fixture and fold its per-group member lists into the
/// per-node shape `SiteSnapshot` wants (xname → groups).
fn prealps_snapshot() -> SiteSnapshot {
  let groups: Vec<Group> =
    serde_json::from_str(include_str!("../testdata/groups-prealps.json"))
      .expect("fixture must parse as GET /groups output");

  let labels: Vec<String> = groups.iter().map(|g| g.label.clone()).collect();

  let mut memberships: BTreeMap<String, Vec<String>> = BTreeMap::new();
  for group in &groups {
    for xname in &group.members.ids {
      memberships
        .entry(xname.clone())
        .or_default()
        .push(group.label.clone());
    }
  }

  SiteSnapshot {
    site: "prealps".to_owned(),
    labels,
    nodes: memberships
      .into_iter()
      .map(|(xname, groups)| NodeMembership { xname, groups })
      .collect(),
  }
}

#[test]
fn prealps_fixture_builds_a_consistent_index() {
  let idx = Index::from_snapshots(vec![prealps_snapshot()]);

  // Single-site fixture: exactly one site, and all 17 groups known.
  assert_eq!(idx.sites().collect::<Vec<_>>(), vec!["prealps"]);
  assert_eq!(idx.groups().count(), 17);

  // Every label resolves to the site.
  for label in idx.groups().map(str::to_owned).collect::<Vec<_>>() {
    assert_eq!(idx.group_to_site(&label), Some("prealps"), "label {label}");
  }

  // Site-umbrella group: full node list of the site.
  let umbrella = idx.group_members("prealps").expect("umbrella group known");
  assert_eq!(umbrella.len(), 39);

  // Empty groups still resolve, with no members.
  for empty in ["cavel_arm", "k3s_agent"] {
    assert_eq!(idx.group_to_site(empty), Some("prealps"), "group {empty}");
    assert_eq!(idx.group_members(empty), Some(&[][..]), "group {empty}");
  }

  // Overlapping membership: one xname in several groups collapses onto
  // the same site, and appears in each group's member list.
  let xname = "x8000c1s5b1n0".to_owned();
  assert_eq!(idx.xname_to_site(&xname), Some("prealps"));
  for group in ["prealps", "rotondo", "cavel", "cavel_gh"] {
    assert!(
      idx
        .group_members(group)
        .expect("group known")
        .contains(&xname),
      "expected {xname} in {group}"
    );
  }
}
