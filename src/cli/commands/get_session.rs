use manta_backend_dispatcher::{error, interfaces::cfs::CfsTrait};

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
) {
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
    .await
    /* .unwrap_or_else(|e| {
      dbg!(&e);
      log::error!("Failed to get CFS sessions. Reason:\n{e}");
      std::process::exit(1);
    }); */
    /* .unwrap_or_else(|e| {
      // dbg!(&e);
      println!("{e}");
      std::process::exit(1);
    }); */
    .unwrap_or_else(|backend_error| {
      // dbg!(&backend_error);
      match backend_error {
        error::Error::SessionNotFound => {
          if let Some(session_name) = session_name_opt {
            println!("Session '{}' could not be found.", session_name);
            std::process::exit(0);
          } else {
            println!("No CFS sessions found.");
            std::process::exit(0);
          }
        }
        _ => {
          log::error!("Failed to get CFS sessions. Reason:\n{backend_error}");
          std::process::exit(1);
        }
      }
    });

  /* let cfs_session_vec = match cfs_session_vec_rslt {
    Ok(sessions) => sessions,
    Err(e) => {
      log::error!("Failed to get CFS sessions. Reason:\n{e}");
      std::process::exit(1);
    }
  }; */

  if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_session_vec).unwrap()
    );
  } else {
    common::cfs_session_utils::print_table_struct(&cfs_session_vec);
  }
}
