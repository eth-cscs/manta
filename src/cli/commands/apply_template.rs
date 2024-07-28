use mesa::bos;

use crate::cli::process::validate_target_hsm_members;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_sessiontemplate_name: &str,
    bos_session_operation: &str,
    limit_opt: Option<&String>,
) {
    //***********************************************************
    // GET DATA
    //
    // Get BOS sessiontemplate
    //
    let bos_sessiontemplate_vec_rslt = bos::template::mesa::http_client::get(
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
    // Validate user has access to the HSM groups and/or xnames in the BOS sessiontemplate
    //
    let target_hsm_vec = bos_sessiontemplate.get_target_hsm();
    let target_xname_vec: Vec<String> = if !target_hsm_vec.is_empty() {
        mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
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
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_xname_vec.clone(),
    )
    .await;

    // Validate user has access to the xnames defined in `limit` argument
    //
    let limit_vec_opt = if let Some(limit) = limit_opt {
        let limit_vec: Vec<String> = limit.split(",").map(|value| value.to_string()).collect();
        let _ = validate_target_hsm_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            limit_vec.clone(),
        )
        .await;

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
    let bos_session = bos::session::shasta::http_client::v2::BosSession {
        // name: Some(bos_sessiontemplate_name.to_string()),
        name: None,
        tenant: None,
        operation: bos::session::shasta::http_client::v2::Operation::from_str(
            bos_session_operation,
        )
        .ok(),
        template_name: bos_sessiontemplate_name.to_string(),
        limit: limit_vec_opt.clone().map(|limit_vec| limit_vec.join(",")),
        stage: Some(false),
        components: None,
        include_disabled: None,
        status: None,
    };

    let create_bos_session_rslt = bos::session::shasta::http_client::v2::post(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos_session,
    )
    .await;

    match create_bos_session_rslt {
        Ok(_) => println!(
            "Template for nodes {:?} updated to '{}'.\nPlease wait a few minutes for CFS batcher to start. Otherwise reboot the nodes manually.",
            limit_vec_opt, bos_sessiontemplate_name
        ),
        Err(e) => eprintln!(
            "ERROR - could not create BOS session. Reason:\n{:#?}.\nExit",
            e
        ),
    }
    // END CREATE BOS SESSION
    //***********************************************************
}
