//! The resolved index and its synchronous lookup surface.
//!
//! The canonical data is the set of `(site, group, members)` triples;
//! the two lookup maps (`group → sites`, `xname → sites`) are derived
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
  /// Every group label accessible at this site (the groups of
  /// `GET /groups`). Includes empty groups.
  pub labels: Vec<String>,
  /// Every node at this site that is a member of an accessible group,
  /// with those groups (the member lists of `GET /groups`, inverted).
  pub nodes: Vec<NodeMembership>,
}

/// One node and the groups it belongs to at a site.
#[derive(Debug, Clone)]
pub struct NodeMembership {
  /// Physical location ID, e.g. `x3000c0s1b0n0`.
  pub xname: String,
  /// Group labels this node is a member of.
  pub groups: Vec<String>,
}

/// A `(group, xname) → sites` routing index.
///
/// Build it with [`crate::refresh`] (live, over HTTP) or
/// [`Index::from_snapshots`] (from data already in hand). All lookups
/// are synchronous and infallible.
///
/// # Collisions
///
/// The index never picks a winner. A group label or xname seen at
/// several sites maps to **all** of them, and the lookups return every
/// site, sorted: empty means unknown, one means resolved, more means
/// ambiguous. Deciding what an ambiguous answer means is the caller's
/// job (manta-server asks the user to name the site). Member lists are
/// kept per `(group, site)`, so they never mix xnames from two sites.
///
/// A site has a group when it lists the label in
/// [`SiteSnapshot::labels`] **or** one of its nodes names it in
/// [`NodeMembership::groups`] ([`crate::refresh`] only produces the
/// latter for listed labels; a hand-built snapshot may not).
#[derive(Debug, Default, Clone)]
pub struct Index {
  /// `group label → sites`, sorted and de-duplicated.
  group_sites: HashMap<String, Vec<String>>,
  /// `xname → sites`, sorted and de-duplicated.
  xname_sites: HashMap<String, Vec<String>>,
  /// `group label → site → sorted member xnames`. `BTreeMap` so
  /// [`Index::groups`] yields labels in a deterministic, sorted order.
  group_members: BTreeMap<String, BTreeMap<String, Vec<String>>>,
  /// All site names seen, sorted and de-duplicated.
  sites: BTreeSet<String>,
}

impl Index {
  /// Fold per-site snapshots into the derived lookup maps. Snapshot
  /// order does not matter.
  ///
  /// This is the HTTP-free construction path: [`crate::refresh`] calls
  /// it after fetching, and callers that already hold the group data
  /// (an embedding process, a fixture-driven test) call it directly.
  pub fn from_snapshots(snapshots: Vec<SiteSnapshot>) -> Self {
    let mut index = Index::default();

    for snap in snapshots {
      index.sites.insert(snap.site.clone());

      for label in snap.labels {
        index
          .group_sites
          .entry(label.clone())
          .or_default()
          .push(snap.site.clone());
        index
          .group_members
          .entry(label)
          .or_default()
          .entry(snap.site.clone())
          .or_default();
      }

      for node in snap.nodes {
        index
          .xname_sites
          .entry(node.xname.clone())
          .or_default()
          .push(snap.site.clone());
        for label in node.groups {
          index
            .group_sites
            .entry(label.clone())
            .or_default()
            .push(snap.site.clone());
          index
            .group_members
            .entry(label)
            .or_default()
            .entry(snap.site.clone())
            .or_default()
            .push(node.xname.clone());
        }
      }
    }

    // Sort + de-duplicate every list for stable output.
    for list in index
      .group_sites
      .values_mut()
      .chain(index.xname_sites.values_mut())
      .chain(
        index
          .group_members
          .values_mut()
          .flat_map(|m| m.values_mut()),
      )
    {
      list.sort();
      list.dedup();
    }

    index
  }

  /// Every site that has a group with this label, sorted. Empty when
  /// the label is unknown; more than one entry is a collision.
  pub fn group_sites(&self, label: &str) -> &[String] {
    self.group_sites.get(label).map_or(&[], Vec::as_slice)
  }

  /// Every site that reported this xname, sorted. Empty when the
  /// xname is unknown; more than one entry is a collision.
  pub fn xname_sites(&self, xname: &str) -> &[String] {
    self.xname_sites.get(xname).map_or(&[], Vec::as_slice)
  }

  /// The members of a group at one site, sorted, or `None` when that
  /// site has no such group.
  ///
  /// A known but empty group returns `Some(&[])`.
  pub fn group_members(&self, label: &str, site: &str) -> Option<&[String]> {
    self
      .group_members
      .get(label)
      .and_then(|by_site| by_site.get(site))
      .map(Vec::as_slice)
  }

  /// Iterate over every site name in the index, sorted.
  pub fn sites(&self) -> impl Iterator<Item = &str> {
    self.sites.iter().map(String::as_str)
  }

  /// Iterate over every known group label, sorted.
  pub fn groups(&self) -> impl Iterator<Item = &str> {
    self.group_members.keys().map(String::as_str)
  }

  /// Iterate over every known xname with its sites.
  ///
  /// Unlike [`Index::groups`] and [`Index::sites`], the order is
  /// **unspecified** — the backing map is a `HashMap`, chosen for O(1)
  /// [`Index::xname_sites`]. Collect into an ordered container when
  /// the output has to be stable (a dump that gets diffed, say).
  pub fn xnames(&self) -> impl Iterator<Item = (&str, &[String])> {
    self
      .xname_sites
      .iter()
      .map(|(xname, sites)| (xname.as_str(), sites.as_slice()))
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
    assert_eq!(idx.group_sites("gpu"), ["alps"]);
    assert_eq!(idx.group_sites("compute_d"), ["daint"]);
    assert!(idx.group_sites("nope").is_empty());
  }

  #[test]
  fn xname_resolves_to_its_site() {
    let idx = sample();
    assert_eq!(idx.xname_sites("x1000c0s0b0n0"), ["alps"]);
    assert_eq!(idx.xname_sites("x2000c0s0b0n0"), ["daint"]);
    assert!(idx.xname_sites("x9999c0s0b0n0").is_empty());
  }

  #[test]
  fn members_are_joined_sorted_and_dedup() {
    let idx = sample();
    assert_eq!(
      idx.group_members("compute", "alps"),
      Some(&["x1000c0s0b0n0".to_owned(), "x1000c0s0b0n1".to_owned()][..]),
    );
    // Node belongs to multiple groups.
    assert_eq!(
      idx.group_members("gpu", "alps"),
      Some(&["x1000c0s0b0n1".to_owned()][..]),
    );
  }

  #[test]
  fn empty_group_is_known_with_no_members() {
    let idx = sample();
    assert_eq!(idx.group_members("empty", "alps"), Some(&[][..]));
    assert_eq!(idx.group_sites("empty"), ["alps"]);
    // Known group, but not at that site.
    assert_eq!(idx.group_members("empty", "daint"), None);
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
    let mut pairs: Vec<_> = idx
      .xnames()
      .map(|(xname, sites)| (xname, sites.join(",")))
      .collect();
    pairs.sort(); // iteration order is unspecified by contract
    assert_eq!(
      pairs,
      vec![
        ("x1000c0s0b0n0", "alps".to_owned()),
        ("x1000c0s0b0n1", "alps".to_owned()),
        ("x2000c0s0b0n0", "daint".to_owned()),
      ]
    );
  }

  #[test]
  fn cross_site_group_collision_keeps_every_site() {
    // Same label "compute" listed at two sites, in either order.
    for order in [["a", "b"], ["b", "a"]] {
      let idx = Index::from_snapshots(
        order
          .iter()
          .map(|site| {
            let xname = format!("x-{site}");
            snapshot(site, &["compute"], vec![node(&xname, &["compute"])])
          })
          .collect(),
      );
      assert_eq!(idx.group_sites("compute"), ["a", "b"]);
      // Members stay per site, never mixed.
      assert_eq!(
        idx.group_members("compute", "a"),
        Some(&["x-a".to_owned()][..])
      );
      assert_eq!(
        idx.group_members("compute", "b"),
        Some(&["x-b".to_owned()][..])
      );
    }
  }

  #[test]
  fn cross_site_xname_collision_keeps_every_site() {
    let idx = Index::from_snapshots(vec![
      snapshot("b", &["g"], vec![node("x1", &["g"])]),
      snapshot("a", &["h"], vec![node("x1", &["h"])]),
    ]);
    assert_eq!(idx.xname_sites("x1"), ["a", "b"]);
    // Distinct labels: each group still resolves to its one site.
    assert_eq!(idx.group_sites("g"), ["b"]);
    assert_eq!(idx.group_sites("h"), ["a"]);
  }

  #[test]
  fn a_bare_node_mention_counts_as_having_the_group() {
    // Site "b" only mentions "shared" via a node's groups (possible in
    // a hand-built snapshot only); that still puts the group at "b".
    let idx = Index::from_snapshots(vec![
      snapshot("a", &["shared"], vec![node("x1", &["shared"])]),
      snapshot("b", &[], vec![node("x2", &["shared"])]),
    ]);
    assert_eq!(idx.group_sites("shared"), ["a", "b"]);
    assert_eq!(
      idx.group_members("shared", "b"),
      Some(&["x2".to_owned()][..])
    );
  }
}
