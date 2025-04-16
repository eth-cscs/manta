use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec_opt: Option<Vec<String>>,
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
        hsm_group_name_vec_opt
    );

    let mut cfs_session_vec = mesa::cfs::session::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        min_age_opt,
        max_age_opt,
        status_opt,
        cfs_session_name_opt,
        None,
    )
    .await
    .unwrap();

    // Retain CFS sessions related to HSM groups
    if let Some(hsm_group_name_vec) = hsm_group_name_vec_opt {
        if !hsm_group_name_vec.is_empty() {
            mesa::cfs::session::mesa::utils::filter_by_hsm(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_session_vec,
                &hsm_group_name_vec,
                limit_number_opt,
                true,
            )
            .await;
        }
    }

    // Retain CFS sessions related to XNAME
    if let Some(xname_vec) = xname_vec_opt {
        mesa::cfs::session::mesa::utils::filter_by_xname(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_session_vec,
            xname_vec.as_slice(),
            limit_number_opt,
            true,
        )
        .await;
    }

    if cfs_session_vec.is_empty() {
        println!("CFS session not found!");
        std::process::exit(0);
    }

    // Validate images in CFS sessions exists in IMS
    // NOTE: do we really care if image exists in IMS or not? we can have the image record
    // in IMS but the file missing in S3, so this validation is not really useful and adds an extra
    // HTTP call whichi is expensive from user's time perspective
    for cfs_session in cfs_session_vec.iter_mut() {
        let cfs_session_name = cfs_session.name.as_ref().unwrap();
        if cfs_session.is_target_def_image() && cfs_session.is_success() {
            log::debug!(
                "Check if Image ID/result_id related to CFS session '{}' exists in IMS",
                cfs_session_name
            );

            let result_id = cfs_session.get_first_result_id().unwrap();

            // Update cfs session result_id if image DOES NOT exists
            if mesa::ims::image::mesa::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                // hsm_group_name_vec,
                Some(&result_id),
            )
            .await
            .is_err()
            {
                cfs_session
                    .status
                    .clone()
                    .unwrap()
                    .artifacts
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                    .result_id = Some("Image missing in IMS".to_string())
            }
        }
    }

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&cfs_session_vec).unwrap()
        );
    } else {
        common::cfs_session_utils::print_table_struct(&cfs_session_vec);
    }
}
