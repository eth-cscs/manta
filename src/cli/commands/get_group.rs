use backend_dispatcher::{interfaces::hsm::group::GroupTrait, types::Group};
use comfy_table::Table;

use crate::backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    auth_token: &str,
    /* base_url: &str,
    root_cert: &[u8], */
    group_name: &str,
    output: &str,
) {
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

    let group = backend.get_group(auth_token, group_name).await.unwrap();

    match output {
        "table" => print_table(group),
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::to_value(group).unwrap()).unwrap()
        ),
        _ => {
            eprintln!("ERROR - output not valid");
            std::process::exit(1);
        }
    }
}

pub fn print_table(group: Group) {
    let mut group_members = group.get_members();
    group_members.sort();

    let mut table = Table::new();

    table.set_header(vec![
        "Group Name",
        "Description",
        "# members",
        "Members",
        "Tags",
    ]);

    table.add_row(vec![
        group.label.clone(),
        group.description.clone().unwrap_or_default(),
        group_members.len().to_string(),
        group_members.join("\n"),
        group.tags.unwrap_or_default().join("\n"),
    ]);

    println!("{table}");
}
