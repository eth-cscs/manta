use mesa::shasta;
use serde_json::{json, Value};

/// Deletes CFS configuration, CFS session, BOS sessiontemplate, BOS session and images related to
/// a CFS configuration. This method is safe. It checks if CFS configuration to delete is assigned
/// to a CFS component as a 'desired configuration' and also checks if image related to CFS
/// configuration is used as a boot image of any node in the system.
pub async fn delete(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name_vec: &Vec<&str>,
    image_id_vec: &Vec<String>,
    cfs_session_value_vec: &Vec<Value>,
    bos_sessiontemplate_value_vec: &Vec<Value>,
) {
    // DELETE DATA
    //
    // DELETE IMAGES
    for image_id in image_id_vec {
        let image_deleted_value =
            shasta::ims::image::http_client::delete(shasta_token, shasta_base_url, &image_id)
                .await
                .unwrap();

        println!(
            "Image deleted: {}",
            image_deleted_value
                .get("id")
                .unwrap_or(&json!("Image not found."))
                .as_str()
                .unwrap()
        );
    }

    // DELETE BOS SESSIONS
    let bos_sessiontemplate_name_vec = bos_sessiontemplate_value_vec
        .iter()
        .map(|bos_sessiontemplate_value| bos_sessiontemplate_value["name"].as_str().unwrap())
        .collect::<Vec<&str>>();

    let bos_session_id_value_vec =
        shasta::bos::session::http_client::get(shasta_token, shasta_base_url, None)
            .await
            .unwrap();

    // Match BOS SESSIONS with the 
    for bos_session_id_value in bos_session_id_value_vec {
        let bos_session_value = shasta::bos::session::http_client::get(
            shasta_token,
            shasta_base_url,
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
            let bos_session_deleted_value = shasta::bos::session::http_client::delete(
                shasta_token,
                shasta_base_url,
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
            log::warn!(
                "Could not find BOS session template related to BOS session {} - Possibly related to a different HSM group or BOS session template was deleted?",
                bos_session_id_value.as_str().unwrap()
            );
        }
    }

    // DELETE CFS SESSIONS
    for cfs_session_value in cfs_session_value_vec {
        let cfs_session_deleted_value = shasta::cfs::session::http_client::delete(
            shasta_token,
            shasta_base_url,
            cfs_session_value["name"].as_str().unwrap(),
        )
        .await
        .unwrap();

        println!(
            "CFS session deleted: {}",
            cfs_session_deleted_value["name"].as_str().unwrap()
        );
    }

    // DELETE BOS SESSIONTEMPLATES
    for bos_sessiontemplate in bos_sessiontemplate_value_vec {
        let bos_sessiontemplate_deleted_value = shasta::bos::template::http_client::delete(
            shasta_token,
            shasta_base_url,
            bos_sessiontemplate["name"].as_str().unwrap(),
        )
        .await
        .unwrap();

        println!(
            "BOS sessiontemplate deleted: {}",
            bos_sessiontemplate_deleted_value["name"].as_str().unwrap()
        );
    }

    // DELETE CFS CONFIGURATIONS
    for cfs_configuration in cfs_configuration_name_vec {
        let cfs_configuration_deleted_value = shasta::cfs::configuration::http_client::delete(
            shasta_token,
            shasta_base_url,
            cfs_configuration,
        )
        .await
        .unwrap();
        println!(
            "CFS configuration deleted: {}",
            cfs_configuration_deleted_value["name"].as_str().unwrap()
        )
    }
}
