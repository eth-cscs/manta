//! Table and JSON renderers for HSM group output.

use comfy_table::{ContentArrangement, Table};
use manta_shared::types::dto::Group;
use nodeset::NodeSet;

/// Print HSM groups as a formatted table.
pub fn print_table(group_vec: &[Group]) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "Group Name",
    "Description",
    "# members",
    "Members",
    "Tags",
  ]);

  for group in group_vec {
    let mut group_members = group.get_members();
    group_members.sort();
    let node_group: NodeSet =
      group_members.join(", ").parse().unwrap_or_default();

    table.add_row(vec![
      group.label.clone(),
      group.description.clone().unwrap_or_default(),
      group_members.len().to_string(),
      node_group.to_string(),
      group.tags.clone().unwrap_or_default().join("\n"),
    ]);
  }

  table.column_mut(3).map(|c| c.set_delimiter(','));

  println!("{table}");
}

#[cfg(test)]
mod tests {
  //! Smoke tests for the HSM group renderer. Two interesting paths:
  //! (a) member list is sorted before joining into a `NodeSet` (a
  //! sort regression would silently re-order the output);
  //! (b) `NodeSet::parse` falls back to `unwrap_or_default` if the
  //! join string is unparseable, so an exotic member list shouldn't
  //! crash the renderer.

  use super::*;
  use serde_json::json;

  fn from_json(value: serde_json::Value) -> Group {
    serde_json::from_value(value).unwrap()
  }

  #[test]
  fn print_empty_list_does_not_panic() {
    print_table(&[]);
  }

  #[test]
  fn print_group_with_no_members_does_not_panic() {
    let g = from_json(json!({
      "label": "empty",
      "description": "no members",
    }));
    print_table(&[g]);
  }

  #[test]
  fn print_group_with_xname_members_renders_via_nodeset() {
    // NodeSet recognizes xname syntax and collapses sequences.
    let g = from_json(json!({
      "label": "compute",
      "description": "compute nodes",
      "members": {
        "ids": ["x3000c0s1b0n0", "x3000c0s1b0n1", "x3000c0s1b0n2"]
      },
    }));
    print_table(&[g]);
  }

  #[test]
  fn print_group_with_unparseable_members_falls_back_to_default() {
    // `NodeSet::parse` fails on arbitrary strings → the
    // `unwrap_or_default()` ensures we still render something.
    let g = from_json(json!({
      "label": "oddballs",
      "members": { "ids": ["foo", "bar", "baz"] },
    }));
    print_table(&[g]);
  }

  #[test]
  fn print_group_with_tags_renders_them_newline_separated() {
    // The tags column joins on '\n'; multi-tag input exercises it.
    let g = from_json(json!({
      "label": "gpu",
      "tags": ["a100", "epyc", "ib-2x"],
      "members": { "ids": ["x3000c0s1b0n0"] },
    }));
    print_table(&[g]);
  }
}
