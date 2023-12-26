use comfy_table::Table;
use mesa::ims::image::{self, r#struct::Image};

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
    let mut image_vec: Vec<Image> = image::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    let image_detail_vec: Vec<(Image, String, String)> = image::utils::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut image_vec,
        hsm_group_name_vec,
        limit_number,
    )
    .await;

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
        table.add_row(vec![
            image_details.0.id.as_ref().unwrap(),
            &image_details.0.name,
            image_details.0.created.as_ref().unwrap(),
            &image_details.1,
            &image_details.2,
        ]);
    }

    println!("{table}");
}
