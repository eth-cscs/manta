use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
};
use anyhow::Error;
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  group_name_arg_opt: Option<&String>,
  settings_hsm_group_name_opt: Option<&String>,
  output: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let group_vec: Vec<Group> = backend
    .get_groups(&shasta_token, Some(&target_hsm_group_vec))
    .await?;

  match output {
    "table" => print_table(&group_vec),
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&serde_json::to_value(group_vec).unwrap())
        .unwrap()
    ),
    _ => {
      return Err(Error::msg("ERROR - output not valid"));
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
