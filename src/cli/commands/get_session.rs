use anyhow::Error;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cli_get_session: &clap::ArgMatches,
) -> Result<(), Error> {
  let hsm_group_name_arg_opt: Option<&String> =
    cli_get_session.get_one("hsm-group");
  let limit_number_opt: Option<&u8> =
    if let Some(true) = cli_get_session.get_one("most-recent") {
      Some(&1)
    } else {
      cli_get_session.get_one::<u8>("limit")
    };
  let xname_vec_arg: Vec<&str> = cli_get_session
    .get_one::<String>("xnames")
    .map(|xname_str| xname_str.split(',').map(|xname| xname.trim()).collect())
    .unwrap_or_default();
  let min_age_opt: Option<&String> = cli_get_session.get_one::<String>("min-age");
  let max_age_opt: Option<&String> = cli_get_session.get_one::<String>("max-age");
  let mut type_opt: Option<String> = cli_get_session.get_one("type").cloned();
  if type_opt == Some("runtime".to_string()) {
    type_opt = Some("dynamic".to_string())
  }
  let status_opt: Option<&String> = cli_get_session.get_one::<String>("status");
  let session_name_opt: Option<&String> =
    cli_get_session.get_one::<String>("name");
  let output_opt: Option<&String> = cli_get_session.get_one("output");

  log::info!("Get CFS sessions",);

  let cfs_session_vec = backend
    .get_and_filter_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_arg_opt.map(|v| vec![v.clone()]).unwrap_or_default(),
      xname_vec_arg,
      min_age_opt,
      max_age_opt,
      type_opt.as_ref(),
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
