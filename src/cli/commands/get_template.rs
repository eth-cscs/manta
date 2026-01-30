use anyhow::Error;
use manta_backend_dispatcher::interfaces::bos::ClusterTemplateTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name_vec: &[String],
  hsm_member_vec: &[String],
  bos_sessiontemplate_name_opt: Option<&str>,
  limit_number_opt: Option<&u8>,
  output: &str,
) -> Result<(), Error> {
  log::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    hsm_group_name_vec
  );

  let mut bos_sessiontemplate_vec = backend
    .get_and_filter_templates(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
      hsm_member_vec,
      bos_sessiontemplate_name_opt,
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
