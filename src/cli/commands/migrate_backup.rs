use std::path::Path;
use mesa::shasta::{bos,cfs,ims};
use mesa::manta;
use std::io;
use std::io::BufWriter;
use std::fs::File;
use std::fs;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos:  Option<&String>,
    destination:  Option<&String>
) {
    println!("Migrate_backup; BOS Template={}, Destination folder={}",bos.unwrap(), destination.unwrap());
    let dest_path = Path::new(destination.unwrap());
    log::debug!("Create directory '{}'", destination.unwrap());
    // TODO control return
    std::fs::create_dir_all(dest_path);
    let _empty_hsm_group_name: Vec<String> = Vec::new();
    let bos_templates = bos::template::http_client::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &_empty_hsm_group_name,
        Option::from(bos.unwrap()),
        None,
    ).await.unwrap_or_default();

    if bos_templates.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        // BOS ------------------------------------------------------------------------------------
        let mut bos_file_path= dest_path.join("bos.json");
        let bos_file = File::create(bos_file_path)
            .expect("bos.json file could not be created.");

        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let bosjson = serde_json::to_writer(&bos_file, &bos_templates[0]);

        // CFS ------------------------------------------------------------------------------------
        let configuration_name  =  &bos_templates[0]["cfs"]["configuration"].to_owned().to_string();
        let mut cn = configuration_name.chars();
        cn.next();
        cn.next_back();
        // cn.as_str();
        let crap = String::from(cn.as_str());
        let cfs_configurations = manta::cfs::configuration::get_configuration(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Option::from(crap).as_ref(),
            &_empty_hsm_group_name,
            Option::from(true),
            None,
        ).await;
        let mut cfs_file_path= dest_path.join("cfs.json");
        let cfs_file = File::create(cfs_file_path)
            .expect("cfs.json file could not be created.");
        // Save to file only the first one returned, we don't expect other BOS templates in the array
        let cfsjson = serde_json::to_writer(&cfs_file, &cfs_configurations[0]);

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

                if ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &_empty_hsm_group_name,
                    Some(&image_id_related_to_bos_sessiontemplate),
                    None,
                    None
                )
                    .await
                    .is_ok()
                {
                    log::info!(
                        "Image ID found related to BOS sessiontemplate {} is {}",
                        bos_templates[0]["boot_sets"]["name"],
                        image_id_related_to_bos_sessiontemplate
                    );

                };
            }
        }
        bos::template::utils::print_table(bos_templates);
    }

    if  let destination = dest_path.is_dir() {
        println!("is directory");
    } else {
        println!("is not directory");
    }
    // Extract in json format:
    //  - the bos-session template
    //  - the cfs configuration referred in the bos-session template
    //  - the contents of the HSM group referred in the bos-session template



    std::process::exit(0);
}
//     if let Some(true) = most_recent {
//         limit_number = Some(&1);
//     } else if let Some(false) = most_recent {
//         limit_number = limit;
//     } else {
//         limit_number = None;
//     }
//
//     let bos_templates = bos::template::http_client::get(
//         shasta_token,
//         shasta_base_url,
//         hsm_group_name,
//         template_name,
//         limit_number,
//     )
//     .await
//     .unwrap_or_default();
//     if bos_templates.is_empty() {
//
//         println!("No BOS template found!");
//         std::process::exit(0);
//     } else {
//         bos::template::utils::print_table(bos_templates);
//     }
// }