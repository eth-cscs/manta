//! The resolved index and its synchronous lookup surface.
//!
//! The canonical data is the set of `(site, group, members)` triples;
//! the two lookup maps (`group → site`, `xname → site`) are derived
//! from it at [`Index::from_snapshots`] time so reads are O(1).

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// One site's contribution to the index.
///
/// This is the HTTP-free seam in front of [`Index::from_snapshots`].
/// [`crate::refresh`] builds it from the group endpoints; a process
/// that already holds the group data (an embedding `manta-server`, a
/// fixture-driven test) constructs it directly and skips HTTP
/// entirely.
#[derive(Debug, Clone)]
pub struct SiteSnapshot {
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
pub struct NodeMembership {
  /// Physical location ID, e.g. `x3000c0s1b0n0`.
  pub xname: String,
  /// Group labels this node is a member of (parsed from the node's
  /// comma-separated `hsm` field).
  pub groups: Vec<String>,
}

/// A resolved `(group, xname) → site` routing index.
///
/// Build it with [`crate::refresh`] (live, over HTTP) or
/// [`Index::from_snapshots`] (from data already in hand). All lookups
/// are synchronous and infallible — they return `None` when nothing is
/// known about the key.
///
/// # Conflict handling
///
/// A principled conflict policy is deferred to ROADMAP Stage 4; Stage 1
/// only needs deterministic, non-panicking behaviour. The deterministic
/// rules when the same key is seen at more than one site (in snapshot
/// order) are:
///
/// - An **xname** resolves to the last site that reported it.
/// - A **group label** listed by a site's `/groups/available` claims
///   the label, replacing any earlier owner. A label known only from
///   nodes' `hsm` fields stays with the first site that mentioned it —
///   a listing outranks a mention.
/// - [`Index::group_members`] reflects the owning site only: when
///   ownership moves, the previous site's members are discarded, and a
///   non-owning site's nodes are never added. Member lists never mix
///   xnames from two sites.
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
  ///
  /// This is the HTTP-free construction path: [`crate::refresh`] calls
  /// it after fetching, and callers that already hold the group data
  /// (an embedding process, a fixture-driven test) call it directly.
  /// Snapshot order matters for cross-site collisions — see "Conflict
  /// handling" above.
  pub fn from_snapshots(snapshots: Vec<SiteSnapshot>) -> Self {
    let mut index = Index::default();

    for snap in snapshots {
      index.sites.insert(snap.site.clone());

      for label in snap.labels {
        // A listed label claims ownership outright. If another site
        // owned it, drop that site's members so the list only ever
        // reflects the owner (see "Conflict handling" above).
        let prev = index.group_to_site.insert(label.clone(), snap.site.clone());
        let members = index.group_members.entry(label).or_default();
        if prev.is_some_and(|p| p != snap.site) {
          members.clear();
        }
      }

      for node in snap.nodes {
        index
          .xname_to_site
          .insert(node.xname.clone(), snap.site.clone());
        for label in node.groups {
          // A node may list a group in `hsm` even if that group wasn't
          // in `/groups/available` (e.g. not directly accessible); such
          // a mention claims the label only while it is unowned, and
          // membership is recorded only when this site owns the label.
          let owner = index
            .group_to_site
            .entry(label.clone())
            .or_insert_with(|| snap.site.clone());
          if *owner == snap.site {
            index
              .group_members
              .entry(label)
              .or_default()
              .push(node.xname.clone());
          }
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

  /// Iterate over every known `(xname, site)` pair.
  ///
  /// Unlike [`Index::groups`] and [`Index::sites`], the order is
  /// **unspecified** — the backing map is a `HashMap`, chosen for O(1)
  /// [`Index::xname_to_site`]. Collect into an ordered container when
  /// the output has to be stable (a dump that gets diffed, say).
  pub fn xnames(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .xname_to_site
      .iter()
      .map(|(xname, site)| (xname.as_str(), site.as_str()))
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
    Index::from_snapshots(vec![
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
  fn xnames_enumerates_every_pair() {
    let idx = sample();
    let mut pairs: Vec<_> = idx.xnames().collect();
    pairs.sort(); // iteration order is unspecified by contract
    assert_eq!(
      pairs,
      vec![
        ("x1000c0s0b0n0", "alps"),
        ("x1000c0s0b0n1", "alps"),
        ("x2000c0s0b0n0", "daint"),
      ]
    );
  }

  #[test]
  fn cross_site_collision_is_last_write_wins() {
    // Same label "compute" listed at two sites; the later snapshot wins.
    let idx = Index::from_snapshots(vec![
      snapshot("a", &["compute"], vec![node("x1", &["compute"])]),
      snapshot("b", &["compute"], vec![node("x2", &["compute"])]),
    ]);
    assert_eq!(idx.group_to_site("compute"), Some("b"));
    // Members follow the owning site — site a's node is discarded, so
    // the group never mixes xnames from two sites.
    assert_eq!(idx.group_members("compute"), Some(&["x2".to_owned()][..]));
    // Each xname still resolves to its own site.
    assert_eq!(idx.xname_to_site("x1"), Some("a"));
    assert_eq!(idx.xname_to_site("x2"), Some("b"));
  }

  #[test]
  fn listed_label_outranks_cross_site_hsm_mention() {
    // Site "a" lists "shared" in /groups/available; site "b" only
    // mentions it via a node's hsm field. The listing keeps ownership
    // and the mentioning site's node is not recorded as a member.
    let idx = Index::from_snapshots(vec![
      snapshot("a", &["shared"], vec![node("x1", &["shared"])]),
      snapshot("b", &[], vec![node("x2", &["shared"])]),
    ]);
    assert_eq!(idx.group_to_site("shared"), Some("a"));
    assert_eq!(idx.group_members("shared"), Some(&["x1".to_owned()][..]));
    // The mentioning site's node itself still routes to its own site.
    assert_eq!(idx.xname_to_site("x2"), Some("b"));
  }
}
