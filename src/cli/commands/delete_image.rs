pub mod command {

    use csm_rs::ims::image::{self, r#struct::Image};

    pub async fn exec(
        shasta_token: &str,
        shasta_base_url: &str,
        shasta_root_cert: &[u8],
        hsm_name_available_vec: Vec<String>,
        image_id_vec: &[&str],
        dry_run: bool,
    ) {
        log::info!(
            "Executing command to delete images: {}",
            image_id_vec.join(", "),
        );

        let mut image_vec: Vec<Image> =
            image::csm_rs::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await
                .unwrap();

        let image_detail_tuple_vec: Vec<(Image, String, String, bool)> = image::utils::filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut image_vec,
            &hsm_name_available_vec,
            None,
        )
        .await;

        // VALIDATE
        // Check images user wants to delete are not being used to boot nodes
        let mut image_xnames_boot_map = Vec::new();
        for image_details_tuple in image_detail_tuple_vec {
            let image_id = image_details_tuple.0.name;
            if image_details_tuple.3 && image_id_vec.contains(&image_id.as_str()) {
                image_xnames_boot_map.push(image_id);
            }
        }

        // Exit if any image id user wants to delete is used to boot nodes
        if !image_xnames_boot_map.is_empty() {
            eprintln!(
                "ERROR - The following images could not be deleted since they boot nodes.\n{:#?}",
                image_xnames_boot_map
            );
            std::process::exit(1);
        }

        for image_id in image_id_vec {
            if dry_run {
                eprintln!("Dry-run enabled. No changes persisted into the system");
            } else {
                let _ = image::shasta::http_client::delete(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &image_id,
                )
                .await;
            }
        }

        println!("Images deleted:\n{:#?}", image_id_vec);
    }
}
