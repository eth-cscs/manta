use comfy_table::{Column, ContentArrangement, Table};
use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  /* base_url: &str,
  root_cert: &[u8], */
  group_name_vec_opt: Option<&[&str]>,
  output: &str,
) -> Result<(), Error> {
  /* let group_backend: hsm::group::r#struct::HsmGroup = hsm::group::http_client::get(
      auth_token,
      base_url,
      root_cert,
      Some(&group_name.to_string()),
  )
  .await
  .unwrap()
  .first()
  .unwrap()
  .clone();

  let group: HsmGroup = group_backend.into(); */

  let group_vec: Vec<Group> = backend
    .get_groups(auth_token, group_name_vec_opt)
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

  // let group_vec = backend.get_group(auth_token, group_name).await.unwrap();

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
