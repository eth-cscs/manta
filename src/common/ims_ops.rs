use crate::common::{bos_sessiontemplate_utils, cfs_session_utils};

pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name: String,
) -> Option<String> {
    log::info!(
        "Searching in CFS sessions for image ID related to CFS configuration '{}'",
        cfs_configuration_name
    );

    // Get all CFS sessions related which has succeeded and built an image related to CFS
    // configuration

    let image_id_opt = cfs_session_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration_name,
    )
    .await;

    if let Some(image_id) = &image_id_opt {
        log::info!(
            "Image with ID '{}' related to CFS configuration '{}' found",
            image_id,
            cfs_configuration_name
        );

        return image_id_opt;
    }

    log::info!("No CFS session related to CFS configuration '{}' found, falling back to BOS sessiontemplate", cfs_configuration_name);
    log::info!(
        "Searching in BOS sessiontemplates for image ID related to CFS configuration '{}'",
        cfs_configuration_name
    );

    bos_sessiontemplate_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration_name,
    )
    .await
}
