use mesa::shasta::cfs;

// TODO: move to mesa
/// Finds image ID linked to a CFS configuration. It supports when image ID recreated or
/// overwritten by SAT command
pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: String,
) -> String {
    let bos_sessiontemplate_list = mesa::shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let bos_sessiontemplate = bos_sessiontemplate_list
        .iter()
        .find(|bos_session_template| {
            bos_session_template
                .pointer("/cfs/configuration")
                .unwrap()
                .as_str()
                .unwrap()
                .eq(&cfs_configuration_name)
        });

    log::debug!("BOS sessiontemplate details:\n{:#?}", bos_sessiontemplate);

    let mut image_id_from_bos_sessiontemplate = "".to_string();

    if bos_sessiontemplate.is_some() {
        for (_boot_sets_param, boot_sets_value) in bos_sessiontemplate.unwrap()["boot_sets"]
            .as_object()
            .unwrap()
        {
            if boot_sets_value.get("path").is_some() {
                image_id_from_bos_sessiontemplate = boot_sets_value["path"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();
                break;
            }
        }
    } else {
        // Get most recent CFS session target image for the node
        let mut cfs_sessions_details = cfs::session::http_client::get(
            shasta_token,
            shasta_base_url,
            None,
            None,
            None,
            Some(true),
        )
        .await
        .unwrap();

        cfs_sessions_details.retain(|cfs_session_details| {
            cfs_session_details["target"]["definition"].eq("image")
                && cfs_session_details["configuration"]["name"].eq(&cfs_configuration_name)
        });

        let cfs_session = cfs_sessions_details.first().unwrap().clone();

        log::debug!("CFS session details:\n{:#?}", cfs_session);

        let cfs_session_status_artifacts_result_id = if !cfs_session["status"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            cfs_session["status"]["artifacts"][0]["result_id"]
                .as_str()
                .unwrap()
                .to_string()
        } else {
            "".to_string()
        };

        image_id_from_bos_sessiontemplate = cfs_session_status_artifacts_result_id;
    }

    image_id_from_bos_sessiontemplate.to_string()
}
