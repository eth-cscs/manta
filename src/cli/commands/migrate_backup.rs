use humansize::DECIMAL;
use std::fs::File;
use std::path::Path;

use mesa::ims::s3::{s3_auth, s3_download_object, s3_get_object_size};
use crate::cli::commands::migrate_restore;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos: Option<&String>,
    destination: Option<&String>,
    prehook: Option<&String>,
    posthook: Option<&String>,
) {

    println!(
        "Migrate backup \n BOS Template: {}\n Destination folder: {}\n Pre-hook: {}\n Post-hook: {}\n",
        bos.unwrap(),
        destination.unwrap(),
        &prehook.unwrap_or(&"none".to_string()),
        &posthook.unwrap_or(&"none".to_string()),
    );
    if prehook.is_some() && !Path::new(&prehook.unwrap()).exists() {
        eprintln!(
            "Error: the defined prehook file {} does not exist.",
            &prehook.unwrap()
        );
        std::process::exit(2);
    }
    if posthook.is_some() && !Path::new(&posthook.unwrap()).exists() {
        eprintln!(
            "Error: the defined posthook file {} does not exist.",
            &posthook.unwrap()
        );
        std::process::exit(2);
    }
    // println!("Migrate backup of the BOS Template image:\n\tBOS file: {}\n\tCFS file: {}\n\tIMS file: {}\n\tHSM file: {}", &bos_file.unwrap(), &cfs_file.unwrap(), &ims_file.unwrap(), &hsm_file.unwrap() );

    let dest_path = Path::new(destination.unwrap());
    let bucket_name = "boot-images";
    let files2download = ["manifest.json", "initrd", "kernel", "rootfs"];
    let files2download_count = files2download.len() + 4; // manifest.json, initrd, kernel, rootfs, bos, cfs, hsm, ims
    log::debug!("Create directory '{}'", destination.unwrap());
    match std::fs::create_dir_all(dest_path) {
        Ok(_ok) => _ok,
        Err(error) => panic!(
            "Unable to create directory {}. Error returned: {}",
            &dest_path.to_string_lossy(),
            error
        ),
    };
    let bos_file_name = String::from(bos.unwrap()) + ".json";
    let bos_file_path = dest_path.join(bos_file_name);

    let hsm_file_name = String::from(bos.unwrap()) + "-hsm.json";
    let hsm_file_path = dest_path.join(hsm_file_name);


    let _empty_hsm_group_name: Vec<String> = Vec::new();
    let mut bos_templates = mesa::bos::template::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos,
    )
    .await
    .unwrap();

    mesa::bos::template::mesa::utils::filter(
        &mut bos_templates,
        &Vec::new(),
        &Vec::new(),
        None,
        None,
    )
    .await;
    let mut download_counter = 1;

    if bos_templates.is_empty() {
        println!("No BOS template found!");
        std::process::exit(1);
    } else {
        // BOS ------------------------------------------------------------------------------------
        let bos_file = File::create(&bos_file_path).expect("bos.json file could not be created.");
        println!(
            "Downloading BOS session template {} to {} [{}/{}]",
            &bos.unwrap(),
            &bos_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download_count
        );

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _bosjson = serde_json::to_writer_pretty(&bos_file, &bos_templates[0]);
        download_counter += 1;

        // HSM group -----------------------------------------------------------------------------

        let hsm_file = File::create(&hsm_file_path).expect("HSM file could not be created.");
        println!(
            "Downloading HSM configuration in bos template {} to {} [{}/{}]",
            &bos.unwrap(),
            &hsm_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download_count
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
        let _hsmjson = serde_json::to_writer_pretty(&hsm_file, &hsm_group_json);

        // CFS ------------------------------------------------------------------------------------
        let configuration_name = &bos_templates[0]
            .cfs
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap()
            .to_owned();
        let cfs_configurations = mesa::cfs::configuration::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&configuration_name),
        )
        .await
        .unwrap();
        let cfs_file_name = String::from(configuration_name.clone().as_str()) + ".json";
        let cfs_file_path = dest_path.join(&cfs_file_name);
        let cfs_file = File::create(&cfs_file_path).expect("cfs.json file could not be created.");
        println!(
            "Downloading CFS configuration {} to {} [{}/{}]",
            // cn.clone().as_str(),
            &configuration_name,
            &cfs_file_path.clone().to_string_lossy(),
            &download_counter,
            &files2download_count
        );

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _cfsjson = serde_json::to_writer_pretty(&cfs_file, &cfs_configurations[0]);
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
                let ims_file_name =
                    String::from(image_id_related_to_bos_sessiontemplate.clone().as_str())
                        + "-ims.json";
                let ims_file_path = dest_path.join(&ims_file_name);
                let ims_file =
                    File::create(&ims_file_path).expect("ims.json file could not be created.");

                println!(
                    "Downloading IMS image record {} to {} [{}/{}]",
                    &image_id_related_to_bos_sessiontemplate,
                    &ims_file_path.clone().to_string_lossy(),
                    &download_counter,
                    &files2download_count
                );
                match mesa::ims::image::shasta::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&image_id_related_to_bos_sessiontemplate),
                )
                .await
                {
                    Ok(ims_record) => {
                        serde_json::to_writer_pretty(&ims_file, &ims_record)
                            .expect("Unable to write new ims record image.json file");
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
                            let object_size = s3_get_object_size(&sts_value, &src, bucket_name)
                                .await
                                .unwrap_or(-1);
                            println!(
                                "Downloading image file {} ({}) to {}/{} [{}/{}]",
                                &src,
                                humansize::format_size(object_size as u64, DECIMAL),
                                &dest,
                                &file,
                                &download_counter,
                                &files2download_count
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
                        } // for file in files2download
                        println!("\nDone, the following image bundle was generated:");
                        println!("\tBOS file: {}", &bos_file_path.to_string_lossy());
                        println!("\tCFS file: {}", &cfs_file_path.to_string_lossy());
                        println!("\tHSM file: {}", &hsm_file_path.to_string_lossy());
                        println!("\tIMS file: {}", &ims_file_path.to_string_lossy());
                        let ims_image_name = migrate_restore::get_image_name_from_ims_file(&ims_file_path.clone().to_string_lossy().to_string());
                        println!("\tImage name: {}", ims_image_name);
                        for file in files2download {
                            let dest = String::from(destination.unwrap());
                            let src = image_id.clone() + "/" + file;
                            println!("\t\tfile: {}/{}", dest, src);
                        }
                    }
                    Err(e) => {
                        panic!(
                            "Image related to BOS session template {} - NOT FOUND. Error: {}",
                            image_id_related_to_bos_sessiontemplate, e
                        );
                    }
                };

            }
        }

    }
    std::process::exit(1);
}
