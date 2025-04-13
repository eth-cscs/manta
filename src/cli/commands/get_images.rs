use backend_dispatcher::{
    error::Error, interfaces::get_images_and_details::GetImagesAndDetailsTrait, types::ims::Image,
};
use chrono::{DateTime, Local};
use comfy_table::Table;

use crate::backend_dispatcher::StaticBackendDispatcher;

/// If filtering by HSM group, then image name must include HSM group name (It assumms each image
/// is built for a specific cluster based on ansible vars used by the CFS session). The reason
/// for this is because CSCS staff deletes all CFS sessions every now and then...
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &[String],
    id_opt: Option<&String>,
    limit_number: Option<&u8>,
) {
    let image_detail_vec_rslt: Result<Vec<(Image, String, String, bool)>, Error> = backend
        .get_images_and_details(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name_vec,
            id_opt,
            limit_number,
        )
        .await;

    let image_detail_vec = match image_detail_vec_rslt {
        Ok(image_detail_vec) => image_detail_vec,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

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
            &image_details
                .0
                .created
                .as_ref()
                .unwrap()
                .parse::<DateTime<Local>>()
                .unwrap()
                .format("%d/%m/%Y %H:%M:%S")
                .to_string(),
            &image_details.1,
            &image_details.2,
        ]);
    }

    println!("{table}");
}
