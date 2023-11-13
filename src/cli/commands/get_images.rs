use comfy_table::Table;
use mesa::shasta::bos;
use serde_json::json;

/// If filtering by HSM group, then image name must include HSM group name (It assumms each image
/// is built for a specific cluster based on ansible vars used by the CFS session). The reason
/// for this is because CSCS staff deletes all CFS sessions every now and then...
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &Vec<String>,
    limit_number: Option<&u8>,
) {
    let image_resp_value_vec = mesa::shasta::ims::image::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name_vec,
        None,
        None,
        limit_number,
    )
    .await
    .unwrap();

    // We need BOS session templates to find an image created by SAT
    let bos_sessiontemplates_value_vec = bos::template::http_client::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name_vec,
        None,
        None,
    )
    .await
    .unwrap();

    // We need CFS sessions to find images without a BOS session template
    let cfs_session_resp_vec = mesa::shasta::cfs::session::http_client::get_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(true),
    )
    .await
    .unwrap();

    let mut image_detail_vec = Vec::new();

    for image_resp in &image_resp_value_vec {
        let image_id = image_resp["id"].as_str().unwrap();
        let image_name = image_resp["name"].as_str().unwrap();
        let creation_time = image_resp["created"].as_str().unwrap();

        let target_group_name_vec: Vec<String>;
        let cfs_configuration: &str;

        /* let bos_sessiontemplate_name;
        let cfs_session_id; */

        let bos_sessiontemplate_value_opt =
            bos_sessiontemplates_value_vec
                .iter()
                .find(|bos_sessiontemplate_value| {
                    bos_sessiontemplate_value
                        .pointer("/boot_sets/compute/path")
                        .unwrap_or(&json!(""))
                        .as_str()
                        .unwrap()
                        .contains(image_id)
                        || bos_sessiontemplate_value
                            .pointer("/boot_sets/uan/path")
                            .unwrap_or(&json!(""))
                            .as_str()
                            .unwrap()
                            .contains(image_id)
                });

        if let Some(bos_sessiontemplate_value) = bos_sessiontemplate_value_opt {
            log::trace!(
                "BOS session template for image id {} found: {}",
                image_id,
                bos_sessiontemplate_value["name"].as_str().unwrap()
            );

            target_group_name_vec = bos_sessiontemplate_value
                .pointer("/boot_sets/compute/node_groups")
                .unwrap_or(&json!([]))
                .as_array()
                .unwrap()
                .iter()
                .map(|target_group| target_group.as_str().unwrap().to_string())
                .collect();

            cfs_configuration = bos_sessiontemplate_value
                .pointer("/cfs/configuration")
                .unwrap()
                .as_str()
                .unwrap();

            /* bos_sessiontemplate_name = bos_sessiontemplate_value["name"].as_str().unwrap();
            cfs_session_id = ""; */
        } else {
            log::trace!(
                "BOS session template for image id {} NOT found. Looking for CFS session",
                image_id
            );

            let cfs_session_image_value_opt = cfs_session_resp_vec.iter().find(|cfs_session| {
                cfs_session
                    .pointer("/status/artifacts/0/result_id")
                    .is_some()
                    && cfs_session
                        .pointer("/status/artifacts/0/result_id")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .eq(image_id)
            });

            if let Some(cfs_session_image_value) = cfs_session_image_value_opt {
                log::trace!(
                    "CFS session for image id {} found: {}",
                    image_id,
                    cfs_session_image_value["id"].as_str().unwrap()
                );

                target_group_name_vec = cfs_session_image_value
                    .pointer("/target/groups")
                    .unwrap()
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|target_group| target_group["name"].as_str().unwrap().to_string())
                    .collect();
                cfs_configuration = cfs_session_image_value_opt
                    .unwrap()
                    .pointer("/configuration/name")
                    .unwrap()
                    .as_str()
                    .unwrap();

                /* bos_sessiontemplate_name = "";
                cfs_session_id = cfs_session_image_value["name"].as_str().unwrap(); */
            } else {
                // Neither BOS session template nor CFS session not found. Most likely there is no BOS session template created and CSCS staff deleted all CFS sessions ...
                target_group_name_vec = vec!["".to_string()];
                cfs_configuration = "";
                /* bos_sessiontemplate_name = "";
                cfs_session_id = ""; */
            }
        }

        let target_groups = target_group_name_vec.join(", ");

        image_detail_vec.push(vec![
            image_id.to_string(),
            image_name.to_string(),
            creation_time.to_string(),
            cfs_configuration.to_string(),
            target_groups.clone(),
            /* bos_sessiontemplate_name.to_string(),
            cfs_session_id.to_string(), */
        ]);
    }

    // Print data
    let mut table = Table::new();

    table.set_header(vec![
        "Image ID",
        "Name",
        "Creation time",
        "CFS configuration",
        "HSM groups",
        // "BOS sessiontemplate",
        // "CFS session name",
    ]);

    for image_details in image_detail_vec {
        table.add_row(image_details);
    }

    println!("{table}");
}
