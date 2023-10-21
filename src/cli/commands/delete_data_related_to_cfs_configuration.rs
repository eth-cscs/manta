use core::time;
use std::thread;

use mesa::shasta;
use serde_json::Value;

/// Deletes CFS configuration, CFS session, BOS sessiontemplate, BOS session and images related to
/// a CFS configuration. This method is safe. It checks if CFS configuration to delete is assigned
/// to a CFS component as a 'desired configuration' and also checks if image related to CFS
/// configuration is used as a boot image of any node in the system.
pub async fn delete(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name_vec: &Vec<&str>,
    image_id_vec: &Vec<String>,
    cfs_session_value_vec: &Vec<Value>,
    bos_sessiontemplate_value_vec: &Vec<Value>,
) {
    // DELETE DATA
    //
    // DELETE IMAGES
    for image_id in image_id_vec {
        let image_deleted_value_rslt = shasta::ims::image::http_client::delete(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            image_id,
        )
        .await;

        // process api response
        match image_deleted_value_rslt {
            Ok(_) => println!("Image deleted: {}", image_id),
            Err(error) => {
                eprintln!("ERROR:\n{:#?}", error);
                let error_response = serde_json::from_str::<Value>(&error.to_string()).unwrap();
                eprintln!("ERROR:\n{:#?}", error_response);
                // std::process::exit(0);
                if error_response["status"].as_u64().unwrap() == 404 {
                    eprintln!("Image {} not found. Continue", image_id);
                }
            }
        }
    }

    // DELETE BOS SESSIONS
    let bos_sessiontemplate_name_vec = bos_sessiontemplate_value_vec
        .iter()
        .map(|bos_sessiontemplate_value| bos_sessiontemplate_value["name"].as_str().unwrap())
        .collect::<Vec<&str>>();

    let bos_session_id_value_vec = shasta::bos::session::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Match BOS SESSIONS with the BOS SESSIONTEMPLATE RELATED
    for bos_session_id_value in bos_session_id_value_vec {
        let bos_session_value = shasta::bos::session::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(bos_session_id_value.as_str().unwrap()),
        )
        .await
        .unwrap();

        if !bos_session_value.is_empty()
            && bos_sessiontemplate_name_vec.contains(
                &bos_session_value.first().unwrap()["templateName"]
                    .as_str()
                    .unwrap(),
            )
        {
            shasta::bos::session::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_session_id_value.as_str().unwrap(),
            )
            .await
            .unwrap();

            println!(
                "BOS session deleted: {}",
                bos_session_id_value.as_str().unwrap() // For some reason CSM API to delete a BOS
                                                       // session does not returns the BOS session
                                                       // ID in the payload...
            );
        } else {
            log::info!(
                "Could not find BOS session template related to BOS session {} - Possibly related to a different HSM group or BOS session template was deleted?",
                bos_session_id_value.as_str().unwrap()
            );
        }
    }

    // DELETE CFS SESSIONS
    let max_attempts = 5;
    for cfs_session_value in cfs_session_value_vec {
        let mut counter = 0;
        loop {
            let deletion_rslt = shasta::cfs::session::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_session_value["name"].as_str().unwrap(),
            )
            .await;

            /* println!(
                "CFS session deleted: {}",
                cfs_session_value["name"].as_str().unwrap()
            ); */
            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS session {} attempt {} of {}, trying again in 2 seconds...", cfs_session_value["name"].as_str().unwrap(), counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
                    "ERROR deleting CFS session {}, please delete it manually.",
                    cfs_session_value["name"].as_str().unwrap(),
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!(
                    "CfS session deleted: {}",
                    cfs_session_value["name"].as_str().unwrap()
                );
                break;
            }
        }
    }

    // DELETE BOS SESSIONTEMPLATES
    let max_attempts = 5;
    for bos_sessiontemplate in bos_sessiontemplate_value_vec {
        let mut counter = 0;
        loop {
            let deletion_rslt = shasta::bos::template::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_sessiontemplate["name"].as_str().unwrap(),
            )
            .await;

            /* match deletion_rslt {
                Ok(_) => println!(
                    "BOS sessiontemplate deleted: {}",
                    bos_sessiontemplate["name"].as_str().unwrap()
                ),
                Err(error) => {
                    let response_error = serde_json::from_str::<Value>(&error.to_string());
                    log::error!("{:#?}", response_error);
                }
            } */

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete BOS sessiontemplate {} attempt {} of {}, trying again in 2 seconds...", bos_sessiontemplate["name"].as_str().unwrap(), counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
                    "ERROR deleting BOS sessiontemplate {}, please delete it manually.",
                    bos_sessiontemplate["name"].as_str().unwrap(),
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!(
                    "BOS sessiontemplate deleted: {}",
                    bos_sessiontemplate["name"].as_str().unwrap()
                );
                break;
            }
        }
    }

    // DELETE CFS CONFIGURATIONS
    let max_attempts = 5;
    for cfs_configuration in cfs_configuration_name_vec {
        let mut counter = 0;
        loop {
            let deletion_rslt = shasta::cfs::configuration::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_configuration,
            )
            .await;

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS configuration {} attempt {} of {}, trying again in 2 seconds...", cfs_configuration, counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
                    "ERROR deleting CFS configuration {}, please delete it manually.",
                    cfs_configuration,
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!("CFS configuration deleted: {}", cfs_configuration);
                break;
            }
        }
    }
}
