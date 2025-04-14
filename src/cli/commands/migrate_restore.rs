use backend_dispatcher::interfaces::bos::ClusterTemplateTrait;
use backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;
use backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use dialoguer::Confirm;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

use crate::backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: Option<&String>,
    cfs_file: Option<&String>,
    hsm_file: Option<&String>,
    ims_file: Option<&String>,
    image_dir: Option<&String>,
    prehook: Option<&String>,
    posthook: Option<&String>,
) {
    println!(
        "Migrate_restore\n Prehook: {}\n Posthook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n HSM_file: {}",
        &prehook.unwrap_or(&"none".to_string()),
        &posthook.unwrap_or(&"none".to_string()),
        bos_file.unwrap(),
        cfs_file.unwrap(),
        ims_file.unwrap(),
        hsm_file.unwrap()
    );
    if prehook.is_some() {
        match crate::common::hooks::check_hook_perms(prehook).await {
            Ok(_) => log::debug!("Pre-hook script exists and is executable."),
            Err(e) => {
                log::error!("{}. File: {}", e, &prehook.unwrap());
                exit(2);
            }
        };
    }
    if posthook.is_some() {
        match crate::common::hooks::check_hook_perms(posthook).await {
            Ok(_) => log::debug!("Post-hook script exists and is executable."),
            Err(e) => {
                log::error!("{}. File: {}", e, &posthook.unwrap());
                exit(2);
            }
        };
    }

    println!();
    if prehook.is_some() {
        println!("Running the pre-hook {}", &prehook.unwrap());
        match crate::common::hooks::run_hook(prehook).await {
            Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                exit(2);
            }
        };
    }

    let migrate_restore_rslt = backend
        .migrate_restore(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            bos_file,
            cfs_file,
            hsm_file,
            ims_file,
            image_dir,
        )
        .await;

    if migrate_restore_rslt.is_err() {
        eprintln!("Error: {}", migrate_restore_rslt.err().unwrap());
        exit(2);
    }

    if posthook.is_some() {
        println!("Running the post-hook {}", &posthook.unwrap());
        match crate::common::hooks::run_hook(posthook).await {
            Ok(_code) => log::debug!("Post-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                exit(2);
            }
        };
    }
    println!("\nDone, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.");

    // ========================================================================================================
}

async fn create_bos_sessiontemplate(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: &String,
    ims_image_id: &String,
) {
    let file_content =
        File::open(bos_file).expect(&format!("Unable to read BOS JSON file '{}'", bos_file));

    let bos_json: BosSessionTemplate = serde_json::from_reader(BufReader::new(file_content))
        .expect("BOS JSON file does not have correct format.");

    let bos_sessiontemplate_name = bos_json.name.unwrap();

    // BOS sessiontemplates need the new ID of the image!
    log::debug!("BOS sessiontemplate name: {}", &bos_sessiontemplate_name);

    let vector = backend
        .get_template(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&bos_sessiontemplate_name),
        )
        .await
        .unwrap_or_else(|error| {
            eprint!(
            "Error: unable to query CSM to get list of BOS sessiontemplates. Error returned: {}",
            error
        );
            std::process::exit(1);
        });

    log::debug!("BOS sessiontemplate filtered: {:#?}", vector);

    if !vector.is_empty() {
        println!("There already exists a BOS sessiontemplate with name '{}'. It can be replaced, but it's dangerous.", &bos_sessiontemplate_name);
        let confirmation = Confirm::new()
            .with_prompt("Do you want to overwrite it?")
            .interact()
            .unwrap();

        if !confirmation {
            println!("Looks like you do not want to continue, bailing out.");
            std::process::exit(2)
        } else {
            // match bos::template::http_client::v2::delete(
            match backend
                .delete_template(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &bos_sessiontemplate_name,
                )
                .await
            {
                Ok(_) => log::debug!(
                    "Ok BOS session template {}, deleted.",
                    &bos_sessiontemplate_name
                ),
                Result::Err(err1) => panic!(
                    "Error, unable to delete BOS session template. Cannot continue. Error: {}",
                    err1
                ),
            };
        }
    }

    let file_content =
        File::open(bos_file).expect(&format!("Unable to read BOS JSON file '{}'", bos_file));

    let mut bos_sessiontemplate: BosSessionTemplate =
        serde_json::from_reader(BufReader::new(file_content))
            .expect("BOS JSON file does not have correct format.");

    // This is as ugly as it can be
    // println!("Path: {}", bos_sessiontemplate.clone().boot_sets.clone().unwrap().compute.clone().unwrap().path.clone().unwrap().to_string());
    let path_modified = format!("s3://boot-images/{}/manifest.json", ims_image_id);
    bos_sessiontemplate
        .boot_sets
        .as_mut()
        .unwrap()
        .get_mut("compute")
        .unwrap()
        .path = Some(path_modified);

    log::debug!("BOS sessiontemplate loaded:\n{:#?}", bos_sessiontemplate);
    log::debug!("BOS sessiontemplate modified:\n{:#?}", &bos_sessiontemplate);

    // match bos::template::http_client::v2::put(
    match backend
        .put_template(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &bos_sessiontemplate,
            &bos_sessiontemplate_name,
        )
        .await
    {
        Ok(_result) => println!(
            "Ok, BOS session template {} created successfully.",
            &bos_sessiontemplate_name
        ),
        Err(e1) => panic!(
            "Error, unable to create BOS sesiontemplate. Error returned by CSM API: {}",
            e1
        ),
    }
}
