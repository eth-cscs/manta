use anyhow::Error;
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
  type_opt: Option<&String>,
  status_opt: Option<&String>,
  session_name_opt: Option<&String>,
  limit_number_opt: Option<&u8>,
  output_opt: Option<&String>,
) -> Result<(), Error> {
  log::info!("Get CFS sessions",);

  let cfs_session_vec = backend
    .get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_available_vec_opt.unwrap_or_default(),
      xname_vec_opt.unwrap_or_default(),
      min_age_opt,
      max_age_opt,
      type_opt,
      status_opt,
      session_name_opt,
      limit_number_opt,
      None,
    )
    .await?;

  if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_session_vec).unwrap()
    );
  } else {
    common::cfs_session_utils::print_table_struct(&cfs_session_vec);
  }

  Ok(())
}
