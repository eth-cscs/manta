use serde_json::Value;

use crate::common::{bos_sessiontemplate_utils, cfs_session_utils};

pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: String,
) -> Option<String> {
    log::info!(
        "Get CFS session related to CFS configuration {}",
        cfs_configuration_name
    );

    // Get all CFS sessions related which has succeeded and built an image related to CFS
    // configuration

    let image_id = cfs_session_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        &cfs_configuration_name,
    )
    .await;

    if image_id.is_some() {
        return image_id;
    }

    /* // Get all CFS sessions which has succeeded
    let cfs_sessions_value_list = mesa::shasta::cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    // Filter CFS sessions to the ones related to CFS configuration and built an image (target
    // definition is 'image' and it actually has at least one artifact)
    let cfs_session_value_target_list =
        cfs_sessions_value_list.iter().filter(|cfs_session_value| {
            cfs_session_value
                .pointer("/configuration/name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
                .eq(&cfs_configuration_name)
                && cfs_session_value
                    .pointer("/target/definition")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq("image")
                && cfs_session_value.pointer("/status/artifacts/0").is_some()
        });

    log::debug!(
        "All CFS sessions related to CFS configuration {}:\n{:#?}",
        cfs_configuration_name,
        cfs_session_value_target_list
    );

    // Find image in CFS sessions
    for cfs_session_value_target in cfs_session_value_target_list {
        log::debug!("CFS session details:'n{:#?}", cfs_session_value_target);

        let cfs_session_name = cfs_session_value_target["name"].as_str().unwrap();

        let image_id = cfs_session_value_target
            .pointer("/status/artifacts/0/result_id")
            .unwrap()
            .as_str();

        log::debug!(
            "Checking image ID {} in CFS session {} exists",
            image_id.unwrap(),
            cfs_session_name
        );

        // Get IMS image related to the CFS session
        if mesa::shasta::ims::image::http_client::get(shasta_token, shasta_base_url, image_id)
            .await
            .is_ok()
        {
            log::info!(
                "Image ID found related to CFS sesison {} is {}",
                cfs_session_name,
                image_id.unwrap()
            );

            return image_id.map(String::from); // from https://users.rust-lang.org/t/convert-option-str-to-option-string/20533/2
        };
    } */

    // No image_id found in CFS sessions, falling back to BOS sessiontemplates

    log::info!("No image ID found based on CFS sessions, falling back to BOS sessiontemplate");

    bos_sessiontemplate_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        &cfs_configuration_name,
    )
    .await

    /* // Get all BOS sessiontemplates
    let bos_sessiontemplate_value_list = mesa::shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Get all BOS sessiontemplates related to CFS configuration
    let bos_sessiontemplate_value_target_list =
        bos_sessiontemplate_value_list
            .iter()
            .filter(|bos_session_template| {
                bos_session_template
                    .pointer("/cfs/configuration")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(&cfs_configuration_name)
            });

    for bos_sessiontemplate_value_target in bos_sessiontemplate_value_target_list {
        log::debug!(
            "BOS sessiontemplate details:\n{:#?}",
            bos_sessiontemplate_value_target
        );

        let bos_sessiontemplate_name = bos_sessiontemplate_value_target["name"].as_str().unwrap();

        for (_boot_sets_param, boot_sets_value) in bos_sessiontemplate_value_target["boot_sets"]
            .as_object()
            .unwrap()
        {
            if let Some(path) = boot_sets_value.get("path") {
                let image_id_related_to_bos_sessiontemplate = path
                    .as_str()
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );

                if ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    Some(&image_id_related_to_bos_sessiontemplate),
                )
                .await
                .is_ok()
                {
                    log::info!(
                        "Image ID found related to BOS sessiontemplate {} is {}",
                        bos_sessiontemplate_name,
                        image_id_related_to_bos_sessiontemplate
                    );

                    return Some(image_id_related_to_bos_sessiontemplate);
                };
            }
        }
    }

    None */

    ///////////////////////// LEGACY ////////////////////////////////////

    /* log::info!(
        "Find images related to BOS sessiontemplate related to CFS configuration {}",
        cfs_configuration_name
    );

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

    // log::debug!("BOS sessiontemplate related to CFS configuration {} found:\n{:#?}", cfs_configuration_name, bos_sessiontemplate_list);

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

                log::debug!(
                    "Image details related to BOS sessiontemplate details:\n{:#?}",
                    image_details_rslt
                );

                if let Ok(image_details) = image_details_rslt {
                    image_detail_list.push(image_details);
                }
            }
        }
    }

    log::info!("Find images related to CFS session");

    // Get CFS session target image for the node
    let mut cfs_sessions_details_resp =
        cfs::session::http_client::get(shasta_token, shasta_base_url, None, None, None, Some(true))
            .await
            .unwrap();

    log::debug!(
        "CFS session details resp:\n{:#?}",
        cfs_sessions_details_resp
    );

    cfs_sessions_details_resp.retain(|cfs_session_details| {
        cfs_session_details["target"]["definition"].eq("image")
            && cfs_session_details
                .pointer("/status/session/succeeded")
                .unwrap()
                .eq("true")
            && cfs_session_details["configuration"]["name"].eq(&cfs_configuration_name)
    });

    if !cfs_sessions_details_resp.is_empty() {
        let cfs_session = cfs_sessions_details_resp.first().unwrap();

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

        log::debug!(
            "Image details related to CFS session:\n{:#?}",
            image_details_rslt
        );

        if let Ok(image_details) = image_details_rslt {
            image_detail_list.push(image_details);
        }
    }

    log::debug!("List of images:\n{:#?}", image_detail_list);

    let most_recent_image = image_detail_list.iter().max_by(|image1, image2| {
        let sort1 = image1["created"].as_str().unwrap();
        let sort2 = image2["created"].as_str().unwrap();
        sort1.cmp(sort2)
    });

    log::debug!("Most recent image created:\n{:#?}", most_recent_image);

    let image_id_related_to_cfs_configuration = if let Some(most_recent_image) = most_recent_image {
        Some(most_recent_image["id"].as_str().unwrap().to_string())
    } else {
        None
    };

    log::info!(
        "Image ID related to CFS configuration {} is {:?}",
        cfs_configuration_name,
        image_id_related_to_cfs_configuration
    );

    image_id_related_to_cfs_configuration */
}

// TODO: move to mesa
/// Finds image ID linked to a CFS configuration. It supports when image ID recreated or
/// overwritten by SAT command.
/// 1. Find most recent CFS session related to CFS configuration and get its resutl_id
/// 2. Find image
/// 2a. If image does not exists, assume image was renamed by a SAT process and fallback to BOS
/// sessiontemplates
/// 3. Find most recent BOS sessiontemplate related to CFS configuration
/// 4. Extract iamge ID from boot_set.path in BOS sessiontemplate
/* pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: String,
) -> Option<String> {
    log::info!(
        "Find images related to BOS sessiontemplate related to CFS configuration {}",
        cfs_configuration_name
    );

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

    // log::debug!("BOS sessiontemplate related to CFS configuration {} found:\n{:#?}", cfs_configuration_name, bos_sessiontemplate_list);

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

                log::debug!(
                    "Image details related to BOS sessiontemplate details:\n{:#?}",
                    image_details_rslt
                );

                if let Ok(image_details) = image_details_rslt {
                    image_detail_list.push(image_details);
                }
            }
        }
    }

    log::info!("Find images related to CFS session");

    // Get CFS session target image for the node
    let mut cfs_sessions_details_resp =
        cfs::session::http_client::get(shasta_token, shasta_base_url, None, None, None, Some(true))
            .await
            .unwrap();

    log::debug!(
        "CFS session details resp:\n{:#?}",
        cfs_sessions_details_resp
    );

    cfs_sessions_details_resp.retain(|cfs_session_details| {
        cfs_session_details["target"]["definition"].eq("image")
            && cfs_session_details
                .pointer("/status/session/succeeded")
                .unwrap()
                .eq("true")
            && cfs_session_details["configuration"]["name"].eq(&cfs_configuration_name)
    });

    if !cfs_sessions_details_resp.is_empty() {
        let cfs_session = cfs_sessions_details_resp.first().unwrap();

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

        log::debug!(
            "Image details related to CFS session:\n{:#?}",
            image_details_rslt
        );

        if let Ok(image_details) = image_details_rslt {
            image_detail_list.push(image_details);
        }
    }

    log::debug!("List of images:\n{:#?}", image_detail_list);

    let most_recent_image = image_detail_list.iter().max_by(|image1, image2| {
        let sort1 = image1["created"].as_str().unwrap();
        let sort2 = image2["created"].as_str().unwrap();
        sort1.cmp(sort2)
    });

    log::debug!("Most recent image created:\n{:#?}", most_recent_image);

    let image_id_related_to_cfs_configuration = if let Some(most_recent_image) = most_recent_image {
        Some(most_recent_image["id"].as_str().unwrap().to_string())
    } else {
        None
    };

    log::info!(
        "Image ID related to CFS configuration {} is {:?}",
        cfs_configuration_name,
        image_id_related_to_cfs_configuration
    );

    image_id_related_to_cfs_configuration
} */

pub async fn get_image_id_from_cfs_session_value(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_session_value: &Value,
) -> Option<String> {
    let cfs_configuration_name = cfs_session_value
        .pointer("/configuration/name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let image_id = cfs_session_utils::get_image_id_from_cfs_session_list(
        shasta_token,
        shasta_base_url,
        &cfs_configuration_name,
        [cfs_session_value.clone()].as_ref(),
    )
    .await;

    if image_id.is_some() {
        return image_id;
    }

    // No image_id found in CFS sessions, falling back to BOS sessiontemplates

    log::info!("No image ID found based on CFS sessions, falling back to BOS sessiontemplate");

    bos_sessiontemplate_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        &cfs_configuration_name,
    )
    .await

    /* let cfs_configuration_name = cfs_session_value
        .pointer("/configuration/name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    log::info!(
        "Find images related to BOS sessiontemplate related to configuration {}",
        cfs_configuration_name
    );

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

    // log::debug!("BOS sessiontemplate related to CFS configuration {} found:\n{:#?}", cfs_configuration_name, bos_sessiontemplate_list);

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

                log::debug!(
                    "Image details related to BOS sessiontemplate details:\n{:#?}",
                    image_details_rslt
                );

                if let Ok(image_details) = image_details_rslt {
                    image_detail_list.push(image_details);
                }
            }
        }
    }

    log::info!("Find images related to most CFS session");

    // Get CFS session related to CFS configuration
    let cfs_sessions_details_resp = [cfs_session_value.clone()];

    if !cfs_sessions_details_resp.is_empty()
        && cfs_session_value
            .pointer("/status/session/succeeded")
            .unwrap()
            .as_str()
            .unwrap()
            .eq("true")
    {
        // Get image IDs related to CFS session
        let cfs_session = cfs_sessions_details_resp.first().unwrap();

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

        log::debug!(
            "Image details related to CFS session:\n{:#?}",
            image_details_rslt
        );

        if let Ok(image_details) = image_details_rslt {
            image_detail_list.push(image_details);
        }
    }

    log::debug!("List of images:\n{:#?}", image_detail_list);

    let most_recent_image = image_detail_list.iter().max_by(|image1, image2| {
        let sort1 = image1["created"].as_str().unwrap();
        let sort2 = image2["created"].as_str().unwrap();
        sort1.cmp(sort2)
    });

    log::debug!("Most recent image created:\n{:#?}", most_recent_image);

    let image_id_related_to_cfs_configuration = if let Some(most_recent_image) = most_recent_image {
        Some(most_recent_image["id"].as_str().unwrap().to_string())
    } else {
        None
    };

    log::info!(
        "Image ID related to CFS configuration {} is {:?}",
        cfs_configuration_name,
        image_id_related_to_cfs_configuration
    );

    image_id_related_to_cfs_configuration */
}
