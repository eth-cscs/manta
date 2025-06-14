use comfy_table::Table;
use csm_rs::bos::template::csm_rs::r#struct::v2::BosSessionTemplate;
use csm_rs::node;

pub fn print_table_struct(bos_sessiontemplate_vec: Vec<BosSessionTemplate>) {
    let mut table = Table::new();

    table.set_header(vec![
        "Name",
        "Image ID",
        "Runtime Configuration",
        "Cfs Enabled",
        "Target",
        "Compute Etag",
    ]);

    for bos_template in bos_sessiontemplate_vec {
        let enable_cfs = bos_template
            .enable_cfs
            .map(|value| value.to_string())
            .unwrap_or("N/A".to_string());

        for boot_set in bos_template.boot_sets.unwrap() {
            let target: Vec<String> = if boot_set.1.node_groups.is_some() {
                // NOTE: very
                // important to
                // define target
                // variable type to
                // tell compiler we
                // want a long live
                // variable
                boot_set.1.node_groups.unwrap()
            } else if boot_set.1.node_list.is_some() {
                boot_set.1.node_list.unwrap()
            } else {
                Vec::new()
            };

            table.add_row(vec![
                bos_template.name.clone().unwrap(),
                boot_set
                    .1
                    .path
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string(),
                bos_template.cfs.clone().unwrap().configuration.unwrap(),
                enable_cfs.clone(),
                node::utils::string_vec_to_multi_line_string(Some(&target), 2),
                boot_set.1.etag.unwrap_or("".to_string()),
            ]);
        }
    }

    println!("{table}");
}

pub async fn get_image_id_related_to_cfs_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name: &String,
) -> Option<String> {
    // Get all BOS sessiontemplates
    let bos_sessiontemplate_value_list = csm_rs::bos::template::csm_rs::http_client::get_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await
    .unwrap();

    get_image_id_from_bos_sessiontemplate_list(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        cfs_configuration_name,
        &bos_sessiontemplate_value_list,
    )
    .await
}

pub async fn get_image_id_from_bos_sessiontemplate_list(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name: &String,
    bos_sessiontemplate_value_list: &[BosSessionTemplate],
) -> Option<String> {
    // Get all BOS sessiontemplates related to CFS configuration
    let bos_sessiontemplate_value_target_list =
        bos_sessiontemplate_value_list
            .iter()
            .filter(|bos_session_template| {
                bos_session_template.cfs.as_ref().is_some_and(|cfs| {
                    cfs.configuration
                        .as_ref()
                        .unwrap()
                        .eq(cfs_configuration_name)
                })
            });

    for bos_sessiontemplate_value_target in bos_sessiontemplate_value_target_list {
        log::debug!(
            "BOS sessiontemplate details:\n{:#?}",
            bos_sessiontemplate_value_target
        );

        let bos_sessiontemplate_name = &bos_sessiontemplate_value_target.name.as_ref().unwrap();

        for boot_sets_value in bos_sessiontemplate_value_target
            .boot_sets
            .as_ref()
            .unwrap()
            .values()
        {
            if let Some(path) = boot_sets_value.path.as_ref() {
                let image_id_related_to_bos_sessiontemplate = path
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );

                if csm_rs::ims::image::shasta::http_client::get_raw(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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

    None
}
