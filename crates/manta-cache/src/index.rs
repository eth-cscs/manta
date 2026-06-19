//! The resolved index and its synchronous lookup surface.
//!
//! The canonical data is the set of `(site, group, members)` triples;
//! the two lookup maps (`group → site`, `xname → site`) are derived
//! from it at [`Index::build`] time so reads are O(1).

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// One site's contribution to the index, as gathered by a refresh.
///
/// This is the HTTP-free seam between [`crate::refresh`] (which builds
/// it from the group endpoints) and [`Index::build`] (which folds it
/// into the lookup maps). Unit tests construct it directly.
#[derive(Debug, Clone)]
pub(crate) struct SiteSnapshot {
  /// Site name (`X-Manta-Site`).
  pub site: String,
  /// Every group label accessible at this site
  /// (from `GET /groups/available`). Includes empty groups.
  pub labels: Vec<String>,
  /// Every accessible node at this site, with its group membership
  /// (from `GET /groups/nodes`).
  pub nodes: Vec<NodeMembership>,
}

/// One node and the groups it belongs to at a site.
#[derive(Debug, Clone)]
pub(crate) struct NodeMembership {
  /// Physical location ID, e.g. `x3000c0s1b0n0`.
  pub xname: String,
  /// Group labels this node is a member of (parsed from the node's
  /// comma-separated `hsm` field).
  pub groups: Vec<String>,
}

/// A resolved `(group, xname) → site` routing index.
///
/// Build it with [`crate::refresh`] (live) or, in tests, from
/// snapshots. All lookups are synchronous and infallible — they return
/// `None` when nothing is known about the key.
///
/// # Conflict handling
///
/// If the same group label or xname is seen at more than one site, the
/// **last** site wins (insertion order over the snapshot slice). A
/// principled conflict policy is deferred to ROADMAP Stage 4; Stage 1
/// only needs deterministic, non-panicking behaviour.
#[derive(Debug, Default, Clone)]
pub struct Index {
  /// `group label → site`.
  group_to_site: HashMap<String, String>,
  /// `xname → site`.
  xname_to_site: HashMap<String, String>,
  /// `group label → sorted member xnames`. `BTreeMap` so
  /// [`Index::groups`] yields labels in a deterministic, sorted order.
  group_members: BTreeMap<String, Vec<String>>,
  /// All site names seen, sorted and de-duplicated.
  sites: BTreeSet<String>,
}

impl Index {
  /// Fold per-site snapshots into the derived lookup maps.
  pub(crate) fn build(snapshots: Vec<SiteSnapshot>) -> Self {
    let mut index = Index::default();

    for snap in snapshots {
      index.sites.insert(snap.site.clone());

      for label in snap.labels {
        index.group_to_site.insert(label.clone(), snap.site.clone());
        // Ensure even empty groups appear in the membership map.
        index.group_members.entry(label).or_default();
      }

      for node in snap.nodes {
        index
          .xname_to_site
          .insert(node.xname.clone(), snap.site.clone());
        for label in node.groups {
          // A node may list a group in `hsm` even if that group wasn't
          // in `/groups/available` (e.g. not directly accessible);
          // record the membership and the routing either way.
          index
            .group_to_site
            .entry(label.clone())
            .or_insert_with(|| snap.site.clone());
          index
            .group_members
            .entry(label)
            .or_default()
            .push(node.xname.clone());
        }
      }
    }

    // Sort + de-duplicate each membership list for stable output.
    for members in index.group_members.values_mut() {
      members.sort();
      members.dedup();
    }

    index
  }

  /// Resolve a group label to the site it lives at.
  pub fn group_to_site(&self, label: &str) -> Option<&str> {
    self.group_to_site.get(label).map(String::as_str)
  }

  /// Resolve a node xname to the site it lives at.
  pub fn xname_to_site(&self, xname: &str) -> Option<&str> {
    self.xname_to_site.get(xname).map(String::as_str)
  }

  /// The members of a group, sorted, or `None` if the label is unknown.
  ///
  /// A known but empty group returns `Some(&[])`.
  pub fn group_members(&self, label: &str) -> Option<&[String]> {
    self.group_members.get(label).map(Vec::as_slice)
  }

  /// Iterate over every site name in the index, sorted.
  pub fn sites(&self) -> impl Iterator<Item = &str> {
    self.sites.iter().map(String::as_str)
  }

  /// Iterate over every known group label, sorted.
  pub fn groups(&self) -> impl Iterator<Item = &str> {
    self.group_members.keys().map(String::as_str)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn node(xname: &str, groups: &[&str]) -> NodeMembership {
    NodeMembership {
      xname: xname.to_owned(),
      groups: groups.iter().map(|g| (*g).to_owned()).collect(),
    }
  }

  fn snapshot(
    site: &str,
    labels: &[&str],
    nodes: Vec<NodeMembership>,
  ) -> SiteSnapshot {
    SiteSnapshot {
      site: site.to_owned(),
      labels: labels.iter().map(|l| (*l).to_owned()).collect(),
      nodes,
    }
  }

  fn sample() -> Index {
    Index::build(vec![
      snapshot(
        "alps",
        &["compute", "gpu", "empty"],
        vec![
          node("x1000c0s0b0n0", &["compute"]),
          node("x1000c0s0b0n1", &["compute", "gpu"]),
        ],
      ),
      snapshot(
        "daint",
        &["compute_d"],
        vec![node("x2000c0s0b0n0", &["compute_d"])],
      ),
    ])
  }

  #[test]
  fn group_resolves_to_its_site() {
    let idx = sample();
    assert_eq!(idx.group_to_site("gpu"), Some("alps"));
    assert_eq!(idx.group_to_site("compute_d"), Some("daint"));
    assert_eq!(idx.group_to_site("nope"), None);
  }

  #[test]
  fn xname_resolves_to_its_site() {
    let idx = sample();
    assert_eq!(idx.xname_to_site("x1000c0s0b0n0"), Some("alps"));
    assert_eq!(idx.xname_to_site("x2000c0s0b0n0"), Some("daint"));
    assert_eq!(idx.xname_to_site("x9999c0s0b0n0"), None);
  }

  #[test]
  fn members_are_joined_sorted_and_dedup() {
    let idx = sample();
    assert_eq!(
      idx.group_members("compute"),
      Some(&["x1000c0s0b0n0".to_owned(), "x1000c0s0b0n1".to_owned()][..]),
    );
    // Node belongs to multiple groups.
    assert_eq!(
      idx.group_members("gpu"),
      Some(&["x1000c0s0b0n1".to_owned()][..]),
    );
  }

  #[test]
  fn empty_group_is_known_with_no_members() {
    let idx = sample();
    assert_eq!(idx.group_members("empty"), Some(&[][..]));
    assert_eq!(idx.group_to_site("empty"), Some("alps"));
  }

  #[test]
  fn sites_are_sorted_and_unique() {
    let idx = sample();
    assert_eq!(idx.sites().collect::<Vec<_>>(), vec!["alps", "daint"]);
  }

  #[test]
  fn groups_are_sorted_and_complete() {
    let idx = sample();
    assert_eq!(
      idx.groups().collect::<Vec<_>>(),
      vec!["compute", "compute_d", "empty", "gpu"]
    );
  }

  #[test]
  fn cross_site_collision_is_last_write_wins() {
    // Same label "compute" at two sites; the later snapshot wins.
    let idx = Index::build(vec![
      snapshot("a", &["compute"], vec![node("x1", &["compute"])]),
      snapshot("b", &["compute"], vec![node("x2", &["compute"])]),
    ]);
    assert_eq!(idx.group_to_site("compute"), Some("b"));
    // Each xname still resolves to its own site.
    assert_eq!(idx.xname_to_site("x1"), Some("a"));
    assert_eq!(idx.xname_to_site("x2"), Some("b"));
  }
}
