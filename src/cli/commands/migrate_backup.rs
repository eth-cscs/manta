use std::path::Path;
use mesa::shasta::{bos, ims};
use mesa::manta;
use std::fs::File;
use mesa::shasta::hsm::http_client::get_hsm_group;
use mesa::shasta::ims::s3::s3::{s3_auth, s3_download_object};
use serde_json::Value;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos:  Option<&String>,
    destination:  Option<&String>
) {
    println!("Migrate_backup; BOS Template={}, Destination folder={}",bos.unwrap(), destination.unwrap());
    let dest_path = Path::new(destination.unwrap());
    let bucket_name = "boot-images";
    let files2download = ["manifest.json", "initrd", "kernel", "rootfs"];
    let mut countfiles2download = files2download.len() + 3 ; // BOS + CFS + HSM
    // let files2download = ["manifest.json"];

    log::debug!("Create directory '{}'", destination.unwrap());
    match std::fs::create_dir_all(dest_path) {
        Ok(_ok) => _ok,
        Err(error) => panic!("Unable to create directory {}. Error returned: {}", &dest_path.to_string_lossy(), error.to_string())
    };
    let mut _empty_hsm_group_name: Vec<String> = Vec::new();
    let bos_templates = bos::template::http_client::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &_empty_hsm_group_name,
        Option::from(bos.unwrap()),
        None,
    ).await.unwrap_or_default();
    let mut download_counter = 1;

    if bos_templates.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        // BOS ------------------------------------------------------------------------------------
        let bos_file_name = String::from(bos.unwrap()) + "-bos.json";
        let bos_file_path= dest_path.join(bos_file_name);
        let bos_file = File::create(&bos_file_path)
            .expect("bos.json file could not be created.");
        println!("Downloading BOS session template {} to {} [{}/{}]", &bos.unwrap(), &bos_file_path.clone().to_string_lossy(), &download_counter, &countfiles2download);

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _bosjson = serde_json::to_writer(&bos_file, &bos_templates[0]);
        download_counter += 1;

        // HSM group -----------------------------------------------------------------------------
        let hsm_file_name = String::from(bos.unwrap()) + "-hsm.json";
        let hsm_file_path= dest_path.join(hsm_file_name);
        let hsm_file = File::create(&hsm_file_path)
            .expect("HSM file could not be created.");
        println!("Downloading HSM configuration in bos template {} to {} [{}/{}]", &bos.unwrap(), &hsm_file_path.clone().to_string_lossy(), &download_counter, &countfiles2download);
        download_counter += 1;

        let hsm_group_name = bos_templates[0]["boot_sets"]["compute"]["node_groups"][0].clone().to_string().replace('\"', "");
        let hsm_group_json = match get_hsm_group(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &hsm_group_name).await {
            Ok(_t) => _t,
            Err(e) => panic!("Error while fetching the HSM configuration description in JSON format: {}", e),
        };

        log::debug!("{:#?}", &hsm_group_json);
        let _hsmjson = serde_json::to_writer(&hsm_file, &hsm_group_json);

        // CFS ------------------------------------------------------------------------------------
        let configuration_name  =  &bos_templates[0]["cfs"]["configuration"].to_owned().to_string();
        let mut cn = configuration_name.chars();
        cn.next();
        cn.next_back();
        // cn.as_str();
        let configuration_name_clean = String::from(cn.as_str());
        let cfs_configurations = manta::cfs::configuration::get_configuration(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Option::from(configuration_name_clean).as_ref(),
            &_empty_hsm_group_name,
            Option::from(true),
            None,
        ).await;
        let cfs_file_name = String::from(cn.clone().as_str()) + "-cfs.json";
        let cfs_file_path= dest_path.join(&cfs_file_name);
        let cfs_file = File::create(&cfs_file_path)
            .expect("cfs.json file could not be created.");
        println!("Downloading CFS configuration {} to {} [{}/{}]", cn.clone().as_str(), &cfs_file_path.clone().to_string_lossy(), &download_counter, &countfiles2download);

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let _cfsjson = serde_json::to_writer(&cfs_file, &cfs_configurations[0]);
        download_counter += 1;

        // Image ----------------------------------------------------------------------------------
        for (_boot_sets_param, boot_sets_value) in bos_templates[0]["boot_sets"]
            .as_object()
            .unwrap()
        {
            if let Some(path) = boot_sets_value.get("path") {
                let image_id_related_to_bos_sessiontemplate = path
                    .as_str()
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );
                _empty_hsm_group_name.push(hsm_group_name.clone().to_string());
                match ims::image::http_client::get(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        &_empty_hsm_group_name,
                        Some(&image_id_related_to_bos_sessiontemplate),
                        None,
                        None
                ).await {
                    Ok(image_value_vec) => {
                        let image_name = &image_value_vec[0]["name"].to_owned().to_string().replace('"',"");
                        let image_id = image_id_related_to_bos_sessiontemplate.clone().to_string();
                        log::info!(
                            "Image ID found related to BOS sessiontemplate {} is {}",
                            &bos.unwrap(),
                            image_id_related_to_bos_sessiontemplate
                        );

                        let ims_file_name = String::from(image_name.clone()) + "-ims.json";
                        let ims_file_path= dest_path.join(&ims_file_name);
                        let ims_file = File::create(&ims_file_path)
                            .expect("ims.json file could not be created.");
                        println!("Downloading IMS file for image with name {} to {} [{}/{}]", &image_name, &ims_file_path.clone().to_string_lossy(), &download_counter, &countfiles2download);

                        // Save to file only the first one returned, we don't expect other BOS templates in the array
                        let _bosjson = serde_json::to_writer(&ims_file, &image_value_vec[0]);

                        let sts_value = match  s3_auth(&shasta_token, &shasta_base_url, &shasta_root_cert).await
                        {
                            Ok(sts_value) => {
                                log::debug!("Debug - STS token:\n{:#?}", sts_value);
                                sts_value
                            }
                            Err(error) => panic!("{}", error.to_string())
                        };
                        for file in  files2download {
                            let dest = String::from(destination.unwrap()) + "/" + &image_name;
                            let src = image_id.clone() + "/" + file;
                            println!("Downloading image file {} to {}/{} [{}/{}]", &src, &dest, &file, download_counter, &countfiles2download);
                            match s3_download_object(&sts_value,
                                                               &src,
                                                               &bucket_name,
                                                               &dest).await {
                                Ok(_result) => {
                                    download_counter += 1;
                                },
                                Err(error) => panic!("Unable to download file {} from s3. Error returned: {}", &src, error.to_string())
                            };
                        }
                        // Here the image should be downloaded already
                    }
                    Err(error) =>  panic!("{}", error.to_string())
                }
            }
        }

        // bos::template::utils::print_table(bos_templates);
    }

    // Extract in json format:
    //  - the contents of the HSM group referred in the bos-session template

    std::process::exit(0);
}
