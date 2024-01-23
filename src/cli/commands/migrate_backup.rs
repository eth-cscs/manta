use std::fs::File;
use std::path::Path;

use mesa::ims::s3::{s3_auth, s3_download_object};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos: Option<&String>,
    destination: Option<&String>,
) {
    println!(
        "Migrate_backup; BOS Template={}, Destination folder={}",
        bos.unwrap(),
        destination.unwrap()
    );
    let dest_path = Path::new(destination.unwrap());
    let bucket_name = "boot-images";
    let files2download = ["manifest.json", "initrd", "kernel", "rootfs"];

    log::debug!("Create directory '{}'", destination.unwrap());
    match std::fs::create_dir_all(dest_path) {
        Ok(_ok) => _ok,
        Err(error) => panic!(
            "Unable to create directory {}. Error returned: {}",
            &dest_path.to_string_lossy(),
            error
        ),
    };

    let _empty_hsm_group_name: Vec<String> = Vec::new();
    let mut bos_templates = mesa::bos::template::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos,
    )
    .await
    .unwrap();

    mesa::bos::template::mesa::utils::filter(&mut bos_templates, &Vec::new(), None, None).await;
    let mut download_counter = 1;

    if bos_templates.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        // BOS ------------------------------------------------------------------------------------

        let bos_file_name = String::from(bos.unwrap()) + ".json";
        let bos_file_path = dest_path.join(bos_file_name);
        let bos_file = File::create(&bos_file_path).expect("bos.json file could not be created.");
        println!(
            "Downloading BOS session template {} to {} [{}/{}]",
            &bos.unwrap(),
            &bos_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download.len() + 3
        );

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _bosjson = serde_json::to_writer(&bos_file, &bos_templates[0]);
        download_counter += 1;

        // HSM group -----------------------------------------------------------------------------
        let hsm_file_name = String::from(bos.unwrap()) + "-hsm.json";

        let hsm_file_path = dest_path.join(hsm_file_name);
        let hsm_file = File::create(&hsm_file_path).expect("HSM file could not be created.");
        println!(
            "Downloading HSM configuration in bos template {} to {} [{}/{}]",
            &bos.unwrap(),
            &hsm_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download.len() + 3
        );
        download_counter += 1;

        let hsm_group_name = bos_templates[0]
            .boot_sets
            .as_ref()
            .unwrap()
            .get("compute")
            .unwrap()
            .node_groups
            .as_ref()
            .unwrap()[0]
            .clone()
            .replace('\"', "");
        let hsm_group_json = match mesa::hsm::group::shasta::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&hsm_group_name),
        )
        .await
        {
            Ok(_t) => _t,
            Err(e) => panic!(
                "Error while fetching the HSM configuration description in JSON format: {}",
                e
            ),
        };

        log::debug!("{:#?}", &hsm_group_json);
        let _hsmjson = serde_json::to_writer(&hsm_file, &hsm_group_json);

        // CFS ------------------------------------------------------------------------------------
        let configuration_name = &bos_templates[0]
            .cfs
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap()
            .to_owned();
        let mut cn = configuration_name.chars();
        cn.next();
        cn.next_back();
        // cn.as_str();
        let configuration_name_clean = String::from(cn.as_str());
        let cfs_configurations = mesa::cfs::configuration::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&configuration_name_clean),
        )
        .await
        .unwrap();
        let cfs_file_name = String::from(cn.clone().as_str()) + ".json";
        let cfs_file_path = dest_path.join(&cfs_file_name);
        let cfs_file = File::create(&cfs_file_path).expect("cfs.json file could not be created.");
        println!(
            "Downloading CFS configuration {} to {} [{}/{}]",
            cn.clone().as_str(),
            &cfs_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download.len() + 2
        );

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _cfsjson = serde_json::to_writer(&cfs_file, &cfs_configurations[0]);
        download_counter += 1;

        // Image ----------------------------------------------------------------------------------
        for boot_sets_value in bos_templates[0].boot_sets.as_ref().unwrap().values() {
            if let Some(path) = &boot_sets_value.path {
                let image_id_related_to_bos_sessiontemplate = path
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );

                if mesa::ims::image::shasta::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&image_id_related_to_bos_sessiontemplate),
                )
                .await
                .is_ok()
                {
                    let image_id = image_id_related_to_bos_sessiontemplate.clone().to_string();
                    log::info!(
                        "Image ID found related to BOS sessiontemplate {} is {}",
                        &bos.unwrap(),
                        image_id_related_to_bos_sessiontemplate
                    );

                    let sts_value =
                        match s3_auth(shasta_token, shasta_base_url, shasta_root_cert).await {
                            Ok(sts_value) => {
                                log::debug!("Debug - STS token:\n{:#?}", sts_value);
                                sts_value
                            }

                            Err(error) => panic!("{}", error.to_string()),
                        };
                    for file in files2download {
                        let dest = String::from(destination.unwrap()) + "/" + &image_id;
                        let src = image_id.clone() + "/" + file;
                        println!(
                            "Downloading image file {} to {}/{} [{}/{}]",
                            &src,
                            &dest,
                            &file,
                            &download_counter,
                            &files2download.len() + 2
                        );

                        match s3_download_object(&sts_value, &src, bucket_name, &dest).await {
                            Ok(_result) => {
                                download_counter += 1;
                            }
                            Err(error) => panic!(
                                "Unable to download file {} from s3. Error returned: {}",
                                &src, error
                            ),
                        };
                    }
                } else {
                    panic!(
                        "Image related to BOS session template {} - NOT FOUND",
                        image_id_related_to_bos_sessiontemplate
                    );
                }
            }
        }

        // bos::template::utils::print_table(bos_templates);
    }

    // Extract in json format:
    //  - the contents of the HSM group referred in the bos-session template

    std::process::exit(0);
}
