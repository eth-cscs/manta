use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  group_name_vec_opt: Option<&[String]>,
  output: &str,
) -> Result<(), Error> {
  let group_vec: Vec<Group> = backend
    .get_groups(auth_token, group_name_vec_opt)
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

  match output {
    "table" => print_table(&group_vec),
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&serde_json::to_value(group_vec).unwrap())
        .unwrap()
    ),
    _ => {
      eprintln!("ERROR - output not valid");
      std::process::exit(1);
    }
  }

  Ok(())
}

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
    let node_group: NodeSet = group_members.join(", ").parse().unwrap();

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
