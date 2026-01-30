use crate::common::authentication::get_api_token;
use crate::common::authorization::get_groups_names_available;
use anyhow::Error;
use manta_backend_dispatcher::interfaces::bos::ClusterTemplateTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cli_get_template: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;
  let name: Option<&String> = cli_get_template.get_one::<String>("name");
  let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");
  let output: &String = cli_get_template
    .get_one("output")
    .expect("ERROR - output must be a valid value");
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    hsm_group_name_arg_opt.unwrap_or(None),
    settings_hsm_group_name_opt,
  )
  .await?;
  let hsm_member_vec = backend
    .get_member_vec_from_group_name_vec(&shasta_token, &target_hsm_group_vec)
    .await?;
  let limit_number_opt = if let Some(limit) = cli_get_template.get_one("limit") {
    Some(limit)
  } else if let Some(true) = cli_get_template.get_one("most-recent") {
    Some(&1)
  } else {
    None
  };

  log::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    target_hsm_group_vec
  );

  let mut bos_sessiontemplate_vec = backend
    .get_and_filter_templates(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      &hsm_member_vec,
      name.map(String::as_str),
      limit_number_opt,
    )
    .await
    .map_err(|e| {
      Error::msg(format!(
        "ERROR - Could not get BOS sessiontemplate list. Reason:\n{:#?}\nExit",
        e
      ))
    })?;

  bos_sessiontemplate_vec.sort_by(|a, b| a.name.cmp(&b.name));

  if bos_sessiontemplate_vec.is_empty() {
    println!("No BOS template found!");
  } else {
    if output == "table" {
      crate::common::bos_sessiontemplate_utils::print_table_struct(
        bos_sessiontemplate_vec,
      );
    } else if output == "json" {
      println!(
        "{}",
        serde_json::to_string_pretty(&bos_sessiontemplate_vec).unwrap()
      );
    }
  }

  Ok(())
}
