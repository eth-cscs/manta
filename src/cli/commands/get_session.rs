use manta_backend_dispatcher::interfaces::cfs::CfsTrait;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name_available_vec_opt: Option<Vec<String>>,
  xname_vec_opt: Option<Vec<&str>>,
  min_age_opt: Option<&String>,
  max_age_opt: Option<&String>,
  status_opt: Option<&String>,
  cfs_session_name_opt: Option<&String>,
  limit_number_opt: Option<&u8>,
  output_opt: Option<&String>,
) {
  log::info!(
    "Get CFS sessions for HSM groups: {:?}",
    hsm_group_name_available_vec_opt
  );

  let cfs_session_vec = backend
    .get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_available_vec_opt,
      xname_vec_opt,
      min_age_opt,
      max_age_opt,
      status_opt,
      cfs_session_name_opt,
      limit_number_opt,
      None,
    )
    .await
    .unwrap_or_else(|e| {
      log::error!("Failed to get CFS sessions. Reason:\n{e}");
      std::process::exit(1);
    });

  if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_session_vec).unwrap()
    );
  } else {
    common::cfs_session_utils::print_table_struct(&cfs_session_vec);
  }
}
