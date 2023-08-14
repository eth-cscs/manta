use mesa::shasta::{cfs, ims};

// TODO: move to mesa
/// Finds image ID linked to a CFS configuration. It supports when image ID recreated or
/// overwritten by SAT command
pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: String,
) -> String {
    let bos_sessiontemplate_list_resp = mesa::shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let bos_sessiontemplate_list =
        bos_sessiontemplate_list_resp
            .iter()
            .filter(|bos_session_template| {
                bos_session_template
                    .pointer("/cfs/configuration")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(&cfs_configuration_name)
            });

    let mut image_detail_list = Vec::new();

    for bos_sessiontemplate in bos_sessiontemplate_list {
        log::debug!("BOS sessiontemplate details:\n{:#?}", bos_sessiontemplate);
        for (_boot_sets_param, boot_sets_value) in
            bos_sessiontemplate["boot_sets"].as_object().unwrap()
        {
            if boot_sets_value.get("path").is_some() {
                let image_id_related_to_bos_sessiontemplate = boot_sets_value["path"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );

                let image_details_rslt = ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    Some(&image_id_related_to_bos_sessiontemplate),
                )
                .await;

                log::debug!("Image details:\n{:#?}", image_details_rslt);

                if let Ok(image_details) = image_details_rslt {
                    image_detail_list.push(image_details);
                }
            }
        }
    }

    // Get most recent CFS session target image for the node
    let mut cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        Some(&1),
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

    log::info!(
        "Get image details for ID {}",
        cfs_session_status_artifacts_result_id
    );

    let image_details_rslt = ims::image::http_client::get(
        shasta_token,
        shasta_base_url,
        Some(&cfs_session_status_artifacts_result_id),
    )
    .await;

    log::debug!("Image details:\n{:#?}", image_details_rslt);

    if let Ok(image_details) = image_details_rslt {
        image_detail_list.push(image_details);
    }

    log::debug!("List of images:\n{:#?}", image_detail_list);

    let most_recent_image = image_detail_list
        .iter()
        .max_by(|image1, image2| {
            let sort1 = image1["created"].as_str().unwrap();
            let sort2 = image2["created"].as_str().unwrap();
            sort1.cmp(sort2)
        })
        .unwrap();

    log::debug!("Most recent image created:\n{:#?}", most_recent_image);

    let image_id_related_to_cfs_configuration =
        most_recent_image["id"].as_str().unwrap().to_string();

    log::info!(
        "Image ID related to CFS configuration {} is {}",
        cfs_configuration_name,
        image_id_related_to_cfs_configuration
    );

    image_id_related_to_cfs_configuration
}
