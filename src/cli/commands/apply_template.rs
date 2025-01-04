use mesa::{bos, hsm, node};

use crate::{
    backend_dispatcher::StaticBackendDispatcher, common::authorization::validate_target_hsm_members,
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session_name_opt: Option<&String>,
    bos_sessiontemplate_name: &str,
    bos_session_operation: &str,
    limit_opt: Option<&String>,
    include_disabled: bool,
    dry_run: bool,
) {
    //***********************************************************
    // GET DATA
    //
    // Get BOS sessiontemplate
    //
    let bos_sessiontemplate_vec_rslt = bos::template::http_client::v2::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(bos_sessiontemplate_name),
    )
    .await;

    let bos_sessiontemplate_vec = match bos_sessiontemplate_vec_rslt {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "ERROR - could not fetch BOS sessiontemplate '{}'. Reason:\n{:#?}\nExit",
                bos_sessiontemplate_name, e
            );
            std::process::exit(1);
        }
    };

    let bos_sessiontemplate = if bos_sessiontemplate_vec.is_empty() {
        eprintln!(
            "ERROR - could not fetch BOS sessiontemplate '{}'\nExit",
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
        hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_vec,
        )
        .await
    } else {
        bos_sessiontemplate.get_target_xname()
    };

    let _ = validate_target_hsm_members(
        &backend,
        shasta_token,
        /* shasta_base_url,
        shasta_root_cert, */
        target_xname_vec.clone(),
    )
    .await;

    // Validate user has access to the xnames defined in `limit` argument
    //
    log::info!("Validate user has access to xnames in BOS sessiontemplate");
    let limit_vec_opt = if let Some(limit) = limit_opt {
        let limit_vec: Vec<String> = limit.split(",").map(|value| value.to_string()).collect();
        let mut xnames_to_validate_access_vec = Vec::new();
        for limit_value in &limit_vec {
            log::info!("Check if limit value '{}', is an xname", limit_value);
            if node::utils::validate_xname_format(limit_value) {
                // limit_value is an xname
                log::info!("limit value '{}' is an xname", limit_value);
                xnames_to_validate_access_vec.push(limit_value.to_string());
            } else if let Some(mut hsm_members_vec) =
                hsm::group::utils::get_member_vec_from_hsm_group_name_opt(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    limit_value,
                )
                .await
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
            /* shasta_base_url,
            shasta_root_cert, */
            xnames_to_validate_access_vec,
        )
        .await;

        log::info!("Access to '{}' granted. Continue.", limit);

        Some(limit_vec)
    } else {
        None
    };

    // END VALIDATION
    //***********************************************************

    //***********************************************************
    // CREATE BOS SESSION
    //
    // Create BOS session request payload
    //
    let bos_session = bos::session::http_client::v2::r#struct::BosSession {
        name: bos_session_name_opt.cloned(),
        tenant: None,
        operation: bos::session::http_client::v2::r#struct::Operation::from_str(
            bos_session_operation,
        )
        .ok(),
        template_name: bos_sessiontemplate_name.to_string(),
        limit: limit_vec_opt.clone().map(|limit_vec| limit_vec.join(",")),
        stage: Some(false),
        components: None,
        include_disabled: Some(include_disabled),
        status: None,
    };

    if dry_run {
        println!("Dry-run enabled. No changes persisted into the system");
        println!("BOS session info:\n{:#?}", bos_session);
    } else {
        let create_bos_session_rslt = bos::session::http_client::v2::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            bos_session,
        )
        .await;

        match create_bos_session_rslt {
             Ok(bos_session) => println!(
                 "BOS session '{}' for BOS sessiontemplate '{}' created.\nPlease wait a few minutes for BOS session to start.",
                 bos_session["name"].as_str().unwrap(), bos_sessiontemplate_name
             ),
             Err(e) => eprintln!(
                 "ERROR - could not create BOS session. Reason:\n{:#?}.\nExit", e),
         }
    }
    // END CREATE BOS SESSION
    //***********************************************************
}
