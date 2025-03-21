use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    cfs::component::shasta::r#struct::v2::{ComponentRequest, ComponentResponse},
    common::jwt_ops,
};

use crate::common::{audit::Audit, kafka::Kafka};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_vec: Vec<String>,
    cfs_session_name: &str,
    dry_run: &bool,
    assume_yes: bool,
    kafka_audit: &Kafka,
) {
    let (
        _cfs_configuration_vec_opt,
        cfs_session_vec_opt,
        _bos_sessiontemplate_vec_opt,
        _image_vec_opt,
        cfs_component_vec_opt,
    ) = mesa::common::utils::get_configurations_sessions_bos_sessiontemplates_images_components(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        true,
        true,
        false,
        false,
        true,
    )
    .await;

    let mut cfs_session_vec = if let Some(cfs_session_vec) = cfs_session_vec_opt {
        cfs_session_vec
    } else {
        eprintln!("ERROR - Problem fetching sessions.");
        std::process::exit(1);
    };

    // Get CFS session to delete
    let cfs_session = cfs_session_vec
        .iter()
        .find(|cfs_session| cfs_session.name.eq(&Some(cfs_session_name.to_string())))
        .expect("CFS session not found")
        .clone();

    // Get CFS configuration related to the CFS session
    let cfs_configuration_name = cfs_session.get_configuration_name().unwrap();

    // Get xnames related to CFS session to delete:
    // - xnames belonging to HSM group related to CFS session
    // - xnames in CFS session
    let xname_vec = if let Some(target_hsm) = cfs_session.get_target_hsm() {
        mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm,
        )
        .await
    } else if let Some(target_xname) = cfs_session.get_target_xname() {
        target_xname
    } else {
        eprintln!("ERROR - neither HSM group nor xnames in CFS session. Exit");
        std::process::exit(1);
    };

    // Validate:
    // - Check CFS session to delete exists
    // - Check CFS session belongs to a cluster the user has access to
    // - CFS configuration related to CFS session is not being used to create an image
    // - CFS configuration related to CFS session is not a desired configuration

    let cfs_session_target_definition = cfs_session.get_target_def().unwrap();

    if cfs_session_target_definition == "image" {
        // Validate CFS session type image:
        // - check CFS configuration related to CFS session is not used to build any other image

        // Get Image ids to delete
        let image_created_by_cfs_configuration = cfs_session.get_result_id_vec();
        if image_created_by_cfs_configuration.len() > 0 {
            if assume_yes {
                // Ask user for confirmation
                let user_msg = format!(
                "Session '{}' used to build images listed below which will get deleted:\n{}\nDo you want to continue?",
                cfs_session_name,
                image_created_by_cfs_configuration.join("\n"),
            );

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(user_msg)
                    .interact()
                    .unwrap()
                {
                    log::info!("Continue",);
                } else {
                    println!("Cancelled by user. Aborting.");
                    std::process::exit(0);
                }
            }

            for image_name in image_created_by_cfs_configuration {
                let _ = mesa::ims::image::shasta::http_client::delete(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &image_name,
                )
                .await;
            }
        }
    } else if cfs_session_target_definition == "dynamic" {
        // Nothing to do ... dynamic CFS sessions will be processed below
    } else {
        eprintln!(
            "CFS session target definition is '{}'. Don't know how to continue. Exit",
            cfs_session_target_definition
        );
        std::process::exit(1);
    };

    // Check if the session to stop belongs to a cluster the user has access
    mesa::cfs::session::mesa::utils::filter_by_hsm(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_session_vec,
        &target_hsm_group_vec,
        None,
        false,
    )
    .await;

    log::info!("Deleting session '{}'", cfs_session_name);

    // DELETE DATA
    //
    // * if session is of type dynamic (runtime session) then:
    // Get retry_policy
    if cfs_session_target_definition == "dynamic" {
        // The CFS session is of type 'target dynamic' (runtime CFS batcher)
        log::info!("CFS session target definition is 'dynamic'.");
        let cfs_global_options = mesa::cfs::component::shasta::http_client::v3::get_options(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await
        .unwrap();

        let retry_policy = cfs_global_options["default_batcher_retry_policy"]
            .as_u64()
            .unwrap();

        // Set CFS components error_count == retry_policy so CFS batcher stops retrying running
        log::info!(
            "Set 'error_count' {} to xnames {:?}",
            retry_policy,
            xname_vec
        );

        // Update CFS component error_count

        let cfs_component_vec: Vec<ComponentResponse> = cfs_component_vec_opt
            .expect("No CFS components")
            .iter()
            .filter(|cfs_component| {
                xname_vec.contains(
                    &cfs_component
                        .id
                        .as_ref()
                        .expect("CFS component found but it has no id???"),
                )
            })
            .cloned()
            .collect();

        // Convert CFS components to another struct we can use for CFS component PUT API
        let mut cfs_component_request_vec = Vec::new();

        // Update CFS component error_count to max value
        for cfs_component in cfs_component_vec {
            let mut cfs_component_request: ComponentRequest = ComponentRequest::from(cfs_component);
            cfs_component_request.error_count = Some(retry_policy);
            cfs_component_request_vec.push(cfs_component_request);
        }

        log::info!(
            "Update error count on nodes {:?} to {}",
            xname_vec,
            retry_policy
        );
        if !dry_run {
            let put_rslt_vec = mesa::cfs::component::shasta::http_client::v2::put_component_list(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_component_request_vec,
            )
            .await;

            for put_rslt in put_rslt_vec {
                if let Err(e) = put_rslt {
                    eprintln!(
                        "ERROR - Could not update error_count on compnents. Reason:\n{}",
                        e
                    );
                }
            }
        } else {
            println!("Update error count on nodes {:?}", xname_vec);
        }
    } else if cfs_session_target_definition == "image" {
        // The CFS session is not of type 'target dynamic' (runtime CFS batcher)
        let image_vec = cfs_session.get_result_id_vec();
        for image_id in image_vec {
            if !dry_run {
                let _ = mesa::ims::image::shasta::http_client::delete(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &image_id,
                )
                .await;
            } else {
                println!(
                    "CFS session target definition is 'image'. Deleting image '{}'",
                    cfs_configuration_name
                );
            }
        }
    } else {
        eprintln!(
            "CFS session target definition is '{}'. Don't know how to continue. Exit",
            cfs_session_target_definition
        );
        std::process::exit(1);
    };

    // Delete CFS session
    log::info!("Delete CFS session '{}'", cfs_session_name);
    if !dry_run {
        let _ = mesa::cfs::session::shasta::http_client::v3::delete(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_session_name,
        )
        .await;
    } else {
        println!("Delete CFS session '{}'", cfs_session_name);
    }

    println!("Session '{cfs_session_name}' has been deleted.");

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "message": format!("delete session '{}'", cfs_session_name)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
