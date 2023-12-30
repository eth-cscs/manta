use mesa::ims::s3::{s3_auth, s3_download_object};
use mesa::{cfs, hsm};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

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
    // let files2download = ["manifest.json"];

    log::debug!("Create directory '{}'", destination.unwrap());
    match std::fs::create_dir_all(dest_path) {
        Ok(_ok) => _ok,
        Err(error) => panic!(
            "Unable to create directory {}. Error returned: {}",
            &dest_path.to_string_lossy(),
            error.to_string()
        ),
    };

    let mut bos_sessiontemplate_vec = mesa::bos::template::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos,
    )
    .await
    .unwrap_or_default();

    mesa::bos::template::mesa::utils::filter(&mut bos_sessiontemplate_vec, &vec![], None, None)
        .await;

    let mut download_counter = 1;

    if bos_sessiontemplate_vec.is_empty() {
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
        let _bosjson = serde_json::to_writer(&bos_file, &bos_sessiontemplate_vec[0]);
        download_counter = download_counter + 1;

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
        download_counter = download_counter + 1;

        let my_hsm_groups_vec = &bos_sessiontemplate_vec[0]
            .boot_sets
            .as_ref()
            .unwrap()
            .get("compute")
            .as_ref()
            .unwrap()
            .node_groups
            .as_ref()
            .unwrap()
            .to_owned();
        let v2: Vec<String> = my_hsm_groups_vec
            .iter()
            .map(|s| s.to_string().replace("\"", ""))
            .collect();

        let mut hsm_map = HashMap::new();

        for v3 in v2 {
            let v4 = vec![v3.clone()];
            let xnames: Vec<String> = hsm::group::shasta::utils::get_member_vec_from_hsm_name_vec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &v4,
            )
            .await;
            hsm_map.insert(v3, xnames);
        }
        // println!("hsm_map={:?}", hsm_map);

        let _hsmjson = serde_json::to_writer(&hsm_file, &hsm_map);

        // CFS ------------------------------------------------------------------------------------
        let configuration_name = &bos_sessiontemplate_vec[0]
            .cfs
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap();
        let mut configuration_name_clean = configuration_name.chars();
        configuration_name_clean.next();
        configuration_name_clean.next_back();
        // cn.as_str();
        let cfs_configurations = cfs::configuration::mesa::http_client::get_and_filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&configuration_name_clean.as_str().to_string()),
            &vec![],
            // Option::from(true),
            Some(&1), // limit 1 means most recent
        )
        .await;
        let cfs_file_name = String::from(configuration_name.as_str()) + ".json";
        let cfs_file_path = dest_path.join(&cfs_file_name);
        let cfs_file = File::create(&cfs_file_path).expect("cfs.json file could not be created.");
        println!(
            "Downloading CFS configuration {} to {} [{}/{}]",
            configuration_name.as_str(),
            &cfs_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download.len() + 2
        );

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _cfsjson = serde_json::to_writer(&cfs_file, &cfs_configurations[0]);
        download_counter = download_counter + 1;

        // Image ----------------------------------------------------------------------------------
        for (_boot_sets_param, boot_sets_value) in
            bos_sessiontemplate_vec[0].boot_sets.as_ref().unwrap()
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

                if mesa::ims::image::mesa::http_client::get(
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
                        match s3_auth(&shasta_token, &shasta_base_url, &shasta_root_cert).await {
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
                        let _result =
                            match s3_download_object(&sts_value, &src, &bucket_name, &dest).await {
                                Ok(_result) => {
                                    download_counter = download_counter + 1;
                                }
                                Err(error) => panic!(
                                    "Unable to download file {} from s3. Error returned: {}",
                                    &src,
                                    error.to_string()
                                ),
                            };
                    }
                    // Here the image should be downloaded already
                };
            }
        }

        // bos::template::utils::print_table(bos_templates);
    }

    // Extract in json format:
    //  - the contents of the HSM group referred in the bos-session template

    std::process::exit(0);
}
