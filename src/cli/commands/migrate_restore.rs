use std::fs;
use std::path::PathBuf;
use mesa::shasta::bos;
use mesa::shasta::hsm::http_client::{create_new_hsm_group, delete_hsm_group};
use derivative::Derivative;
use mesa::shasta::hsm::{HsmGroup, Member};
use dialoguer::Confirm;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file:  Option<&String>,
    cfs_file:  Option<&String>,
    hsm_file:  Option<&String>
) {
    log::info!("Migrate_restore; BOS_file={}, CFS_file={}, HSM_file={}",bos_file.unwrap(), cfs_file.unwrap(), hsm_file.unwrap());
    // IMAGE =======================================================================================
    // Taken from https://cray-hpe.github.io/docs-csm/en-13/operations/image_management/import_external_image_to_ims/
    // Create image record in IMS ------------------------------------------------------------------

    // Create manifest.json ------------------------------------------------------------------------

    // Upload rootfs, initrd, kernel and manifest.json to s3 ---------------------------------------

    // Update IMS image record with link to manifest file in s3 ------------------------------------

    // HSM group ===================================================================================
    // HSM needs to go before CFS, as CFS and BOS have references to it

    // load into memory
    let hsm_data = fs::read_to_string(PathBuf::from(hsm_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let hsm_json: serde_json::Value = serde_json::from_str(&hsm_data)
        .expect("HSM JSON file does not have correct format.");

    // Create new HSM group if not existing

    // Parse HSM group file
    // The file looks like this: {"gele":["x1001c7s1b1n1","x1001c7s1b0n0","x1001c7s1b1n0","x1001c7s1b0n1"]}
    let mut hsm :HsmGroup = serde_json::from_str(hsm_data.as_str()).unwrap();
    log::debug!("HSM group to create {:#?}", &hsm_data.as_str());


    // let exclusive:bool = false; // Make sure this is false, so we can test this without impacting other HSM groups
    // // the following xnames are part of HSM group "gele"
    // let xnames:Vec<String> = vec!["x1001c7s1b0n0".to_string(),
    //                               "x1001c7s1b0n1".to_string(),
    //                               "x1001c7s1b1n0".to_string(),
    //                               "x1001c7s1b1n1".to_string()];
    // let description = "Test group created by function mesa test_1_hsm";
    // let tags:Vec<String> = vec!["dummyTag1".to_string(), "dummyTag2".to_string()];
    // // let tags= vec![]; // sending an empty vector works
    // let hsm_group_name_opt = "manta_created_hsm".to_string();
    if hsm.tags.is_none() {
        hsm.tags = vec![].into();
    }
    if hsm.exclusiveGroup.is_none() {
        hsm.exclusiveGroup = Some(false.to_string());
    }
    // This couldn't be uglier, I know
    let mut hsm2 :HsmGroup = hsm.clone();

        // Create the HSM group
    match create_new_hsm_group(&shasta_token,
                                   &shasta_base_url,
                                   &shasta_root_cert,
                                   &hsm.label,
                                   &hsm.members.unwrap().ids.unwrap(),
                                   &hsm.exclusiveGroup.unwrap(),
                                   &hsm.description.unwrap(),
                                   &hsm.tags.unwrap()).await {
        Ok(_json) => assert!(true),
        Err(error) => {
            if error.to_string().to_lowercase().contains("409") {
                println!("The HSM group {} already exists, it is possible to recreate it, but is a dangerous operation", &hsm.label);
                log::error!("Error message {}", error);
                let confirmation = Confirm::new()
                    .with_prompt("Do you want to recreate it?")
                    .interact()
                    .unwrap();

                if confirmation {
                    println!("Looks like you want to continue");
                    match delete_hsm_group(&shasta_token,
                                           &shasta_base_url,
                                           &shasta_root_cert,
                                           &hsm.label).await {
                        Ok(_) => {
                            // try creating the group again
                            match create_new_hsm_group(&shasta_token,
                                                       &shasta_base_url,
                                                       &shasta_root_cert,
                                                       &hsm2.label,
                                                       &hsm2.members.unwrap().ids.unwrap(),
                                                       &hsm2.exclusiveGroup.unwrap(),
                                                       &hsm2.description.unwrap(),
                                                       &hsm2.tags.unwrap()).await {
                                Ok(_json) => assert!(true),
                                Err(e2) => {
                                    log::error!("Error message {}", e2);
                                    assert!(false,"Second error creating a new HSM group. Bailing out. Error returned: '{}'", e2)
                                }
                            }
                        },
                        Err(e1) => {
                            log::error!("Error message {}", e1);
                            assert!(false,"Error deleting the HSM group {}. Error returned: '{}'", &hsm.label, e1)
                        }
                    }
                } else {
                    assert!(false,"Not deleting the group, cannot continue. Original return error message:");
                }
            }
            // assert!(false,"Error creating a new HSM group. Error returned: '{}'", error)
        },
    };

    // CFS =========================================================================================

    // load into memory
    let cfs_data = fs::read_to_string(PathBuf::from(cfs_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let cfs_json: serde_json::Value = serde_json::from_str(&cfs_data)
        .expect("HSM JSON file does not have correct format.");
    // CFS needs to be cleaned up when loading into the system, the filed lastUpdate should not exist

    // create or update CFS config

    // BOS =========================================================================================

    // load into memory
    let bos_data = fs::read_to_string(PathBuf::from(&bos_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let bos_json: serde_json::Value = serde_json::from_str(&bos_data)
        .expect("HSM JSON file does not have correct format.");

    log::debug!("Migrate_restore: all JSON files loaded ok");



    //
    // let bos_templates = bos::template::http_client::get(
    //     shasta_token,
    //     shasta_base_url,
    //     hsm_group_name,
    //     template_name,
    //     limit_number,
    // )
    //     .await
    //     .unwrap_or_default();

    // if bos_templates.is_empty() {
    //     println!("No BOS template found!");
    //     std::process::exit(0);
    // } else {
    //     bos::template::utils::print_table(bos_templates);
    // }
}