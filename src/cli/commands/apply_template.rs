use manta_backend_dispatcher::{
  interfaces::{
    bos::{ClusterSessionTrait, ClusterTemplateTrait},
    hsm::group::GroupTrait,
  },
  types::bos::session::{BosSession, Operation},
};

use crate::{
  common::{
    authorization::validate_target_hsm_members, node_ops::validate_xname_format,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use dialoguer::{theme::ColorfulTheme, Confirm};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  bos_session_name_opt: Option<&String>,
  bos_sessiontemplate_name: &str,
  bos_session_operation: &str,
  limit: &String,
  include_disabled: bool,
  assume_yes: bool,
  dry_run: bool,
) {
  //***********************************************************
  // GET DATA
  //
  // Get BOS sessiontemplate
  //
  let bos_sessiontemplate_vec_rslt = backend
    .get_and_filter_templates(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &vec![],
      &[],
      Some(&bos_sessiontemplate_name.to_string()),
      None,
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

  let bos_sessiontemplate = if bos_sessiontemplate_vec.is_empty() {
    eprintln!(
      "ERROR - No BOS sessiontemplate '{}' found\nExit",
      bos_sessiontemplate_name
    );
    std::process::exit(1);
  } else {
    bos_sessiontemplate_vec.first().unwrap()
  };

  // END GET DATA
  //***********************************************************

  //***********************************************************
  // VALIDATION
  //
  log::info!("Start BOS sessiontemplate validation");

  // Validate user has access to the BOS sessiontemplate targets (either HSM groups or xnames)
  //
  log::info!("Validate user has access to HSM group in BOS sessiontemplate");
  let target_hsm_vec = bos_sessiontemplate.get_target_hsm();
  let target_xname_vec: Vec<String> = if !target_hsm_vec.is_empty() {
    backend
      .get_member_vec_from_group_name_vec(shasta_token, target_hsm_vec)
      .await
      .unwrap()
    /* hsm::group::utils::get_member_vec_from_hsm_name_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_hsm_vec,
    )
    .await */
  } else {
    bos_sessiontemplate.get_target_xname()
  };

  let _ =
    validate_target_hsm_members(&backend, shasta_token, &target_xname_vec)
      .await;

  // Validate user has access to the xnames defined in `limit` argument
  //
  log::info!("Validate user has access to xnames in BOS sessiontemplate");
  let limit_vec: Vec<String> =
    limit.split(",").map(|value| value.to_string()).collect();
  let mut xnames_to_validate_access_vec = Vec::new();
  for limit_value in &limit_vec {
    log::info!("Check if limit value '{}', is an xname", limit_value);
    if validate_xname_format(limit_value) {
      // limit_value is an xname
      log::info!("limit value '{}' is an xname", limit_value);
      xnames_to_validate_access_vec.push(limit_value.to_string());
    } else if let Some(mut hsm_members_vec) = backend
      .get_member_vec_from_group_name_vec(
        shasta_token,
        vec![limit_value.to_string()],
      )
      .await
      .ok()
    {
      // limit_value is an HSM group
      log::info!(
        "Check if limit value '{}', is an HSM group name",
        limit_value
      );

      xnames_to_validate_access_vec.append(&mut hsm_members_vec);
    } else {
      // limit_value neither is an xname nor an HSM group
      panic!(
          "Value '{}' in 'limit' argument does not match an xname or a HSM group name.",
          limit_value
        );
    }
  }

  log::info!("Validate list of xnames translated from 'limit argument'");

  let _ = validate_target_hsm_members(
    &backend,
    shasta_token,
    &xnames_to_validate_access_vec,
  )
  .await;

  log::info!("Access to '{}' granted. Continue.", limit);

  // END VALIDATION
  //***********************************************************

  //***********************************************************
  // ASK USER FOR CONFIRMATION
  //

  if !assume_yes {
    let operation = if bos_session_operation.to_lowercase() == "boot" {
      "reboot (if necessary)"
    } else {
      bos_session_operation
    };

    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt(format!(
        "{:?}\nThe nodes above will {}. Please confirm to proceed?",
        limit_vec.clone().join(","),
        operation
      ))
      .interact()
      .unwrap()
    {
      log::info!("Continue",);
    } else {
      println!("Cancelled by user. Aborting.");
      std::process::exit(0);
    }
  }

  //***********************************************************
  // CREATE BOS SESSION
  //
  // Create BOS session request payload
  //
  let bos_session = BosSession {
    name: bos_session_name_opt.cloned(),
    tenant: None,
    operation: Operation::from_str(bos_session_operation).ok(),
    template_name: bos_sessiontemplate_name.to_string(),
    limit: Some(limit_vec.clone().join(",")),
    stage: Some(false),
    components: None,
    include_disabled: Some(include_disabled),
    status: None,
  };

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!("BOS session info:\n{:#?}", bos_session);
  } else {
    let create_bos_session_rslt = backend
      .post_template_session(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos_session.into(),
      )
      .await;

    match create_bos_session_rslt {
             Ok(bos_session) => println!(
                 "BOS session '{}' for BOS sessiontemplate '{}' created.\nPlease wait a few minutes for BOS session to start.",
                 bos_session.name.unwrap(), bos_sessiontemplate_name
             ),
             Err(e) => eprintln!(
                 "ERROR - could not create BOS session. Reason:\n{:#?}.\nExit", e),
         }
  }
  // END CREATE BOS SESSION
  //***********************************************************
}
