use crate::common::{bos_sessiontemplate_utils, cfs_session_utils};

pub async fn get_image_id_from_cfs_configuration_name(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
        shasta_root_cert,
        &cfs_configuration_name,
    )
    .await;

    if image_id.is_some() {
        return image_id;
    }

    log::info!("No image ID found based on CFS sessions, falling back to BOS sessiontemplate");

    bos_sessiontemplate_utils::get_image_id_related_to_cfs_configuration(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration_name,
    )
    .await
}
