use manta_backend_dispatcher::interfaces::bos::ClusterTemplateTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name_vec: &Vec<String>,
  hsm_member_vec: &[String],
  bos_sessiontemplate_name_opt: Option<&String>,
  limit_number_opt: Option<&u8>,
  output: &str,
) {
  log::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    hsm_group_name_vec
  );

  let bos_sessiontemplate_vec_rslt = backend
    .get_and_filter_templates(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
      hsm_member_vec,
      bos_sessiontemplate_name_opt,
      limit_number_opt,
    )
    .await;

  let bos_sessiontemplate_vec = match bos_sessiontemplate_vec_rslt {
    Ok(bos_sessiontemplate_vec) => bos_sessiontemplate_vec,
    Err(e) => {
      eprintln!(
                "ERROR - Could not fetch BOS sessiontemplate list. Reason:\n{:#?}\nExit",
                e
            );
      std::process::exit(1);
    }
  };

  if bos_sessiontemplate_vec.is_empty() {
    println!("No BOS template found!");
    std::process::exit(0);
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
}
