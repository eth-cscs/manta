use mesa::shasta;

use serde_json::Value;

use crate::common::{self, ims_ops};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    session_name: Option<&String>,
    limit_number: Option<&u8>,
    output_opt: Option<&String>,
) {
    let mut cfs_session_value_list = shasta::cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        session_name,
        limit_number,
        None,
    )
    .await
    .unwrap_or_default();

    log::debug!("CFS sessions:\n{:#?}", cfs_session_value_list);

    if cfs_session_value_list.is_empty() {
        println!("CFS session not found!");
        std::process::exit(0);
    } else {
        let cfs_configuration_name = cfs_session_value_list
            .first()
            .unwrap()
            .pointer("/configuration/name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let mut cfs_session_list = Vec::<Value>::new();

        for cfs_session_value in cfs_session_value_list.iter_mut() {
            log::debug!("CFS session:\n{:#?}", cfs_session_value);

            let cfs_session_name = cfs_session_value["name"].as_str().unwrap().to_string();

            if cfs_session_value
                .pointer("/target/definition")
                .unwrap()
                .as_str()
                .eq(&Some("image"))
                && cfs_session_value
                    .pointer("/status/session/succeeded")
                    .unwrap()
                    .as_str()
                    .eq(&Some("true"))
            {
                log::info!(
                    "Find image ID related to CFS configuration {} in CFS session {}",
                    cfs_configuration_name,
                    cfs_session_name
                );

                let new_image_id_opt = ims_ops::get_image_id_from_cfs_session_value(
                    shasta_token,
                    shasta_base_url,
                    cfs_session_value,
                )
                .await;

                log::info!("Image ID found: {:?}", new_image_id_opt);

                if let Some(new_image_id) = new_image_id_opt {
                    if cfs_session_value
                        .pointer("/status/artifacts/0/result_id")
                        .is_some()
                    {
                        /* let cfs_session: cfs_session_utils::CfsSession =
                            serde_json::from_value(cfs_session_value.clone()).unwrap();

                        println!("CFS SESSION STRUCT:\n{:#?}", cfs_session);

                        cfs_session.status.unwrap().actifacts.unwrap().first().unwrap().result_id = Some(new_image_id); */

                        // let mut cfs_session_value_cloned = cfs_session_value.clone();

                        if let Value::String(current_image_id) = &mut cfs_session_value
                            .pointer_mut("/status/artifacts/0/result_id")
                            .unwrap()
                        {
                            log::info!(
                                "Update image ID from {} to {}",
                                current_image_id,
                                new_image_id
                            );

                            *current_image_id = new_image_id;

                            log::debug!("New CFS session details:\n{:#?}", cfs_session_value);
                        }

                        cfs_session_list.push(cfs_session_value.clone());
                    } else {
                        cfs_session_list.push(cfs_session_value.clone());
                    }
                } else {
                    cfs_session_list.push(cfs_session_value.clone());
                }
            } else {
                cfs_session_list.push(cfs_session_value.clone());
            }

            /* let cfs_session_value_aux = cfs_session_value.clone();

            let image_id_opt = common::ims_ops::get_image_id_from_cfs_session_value(
                shasta_token,
                shasta_base_url,
                cfs_session_value_aux.clone(),
            )
            .await;

            if let Some(image_id) = image_id_opt {
                if cfs_session_value_aux
                    .pointer("/status/artifacts/0/result_id")
                    .is_some()
                {
                    let result_id = &mut cfs_session_value_aux
                        .pointer("/status/artifacts/0/result_id")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();

                    *result_id = image_id;

                    cfs_session_list.push(cfs_session_value_aux);
                } else {
                    cfs_session_list.push(cfs_session_value_aux);
                }
            } */
        }
        if output_opt.is_some() && output_opt.unwrap().eq("json") {
            println!(
                "{}",
                serde_json::to_string_pretty(&cfs_session_value_list).unwrap()
            );
        } else {
            common::cfs_session_utils::print_table(&cfs_session_value_list);
        }
    }

    // LEGACY

    /* let cfs_session_table_data_list = manta::cfs::session::get_sessions(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        session_name,
        limit_number,
    )
    .await;

    cfs::session::utils::print_table(cfs_session_table_data_list); */
}
