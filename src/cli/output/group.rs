use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::Group;
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
