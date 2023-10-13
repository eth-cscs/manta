use mesa::shasta;

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    output_opt: Option<&String>,
) {
    let mut cfs_session_vec = mesa::mesa::cfs::session::http_client::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
    )
    .await
    .unwrap();

    if cfs_session_vec.is_empty() {
        println!("CFS session not found!");
        std::process::exit(0);
    }

    for cfs_session in cfs_session_vec.iter_mut() {
        log::debug!("CFS session:\n{:#?}", cfs_session);

        if cfs_session
            .target
            .as_ref()
            .unwrap()
            .definition
            .as_ref()
            .unwrap()
            .eq("image")
            && cfs_session
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .succeeded
                .as_ref()
                .unwrap()
                .eq("true")
        {
            log::info!(
                "Find image ID related to CFS configuration {} in CFS session {}",
                cfs_session
                    .configuration
                    .as_ref()
                    .unwrap()
                    .name
                    .as_ref()
                    .unwrap(),
                cfs_session.name.as_ref().unwrap()
            );

            let new_image_id_opt = if cfs_session
                .status
                .as_ref()
                .and_then(|status| {
                    status.artifacts.as_ref().and_then(|artifacts| {
                        artifacts
                            .first()
                            .and_then(|artifact| artifact.result_id.clone())
                    })
                })
                .is_some()
            {
                let cfs_session_image_id = cfs_session
                    .status
                    .as_ref()
                    .unwrap()
                    .artifacts
                    .as_ref()
                    .unwrap()
                    .first()
                    .unwrap()
                    .result_id
                    .as_deref();
                let new_image_id_vec = shasta::ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    hsm_group_name_opt,
                    cfs_session_image_id,
                    None,
                )
                .await
                .unwrap();
                let new_image_id = new_image_id_vec.first().unwrap();

                Some(new_image_id.as_str().unwrap().to_string())
            } else {
                None
            };

            if new_image_id_opt.is_some() {
                cfs_session
                    .status
                    .clone()
                    .unwrap()
                    .artifacts
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                    .result_id = new_image_id_opt;
            }
        }
    }

    cfs_session_vec = mesa::mesa::cfs::session::http_client::utils::filter(
        shasta_token,
        shasta_base_url,
        &mut cfs_session_vec,
        hsm_group_name_opt,
        cfs_session_name_opt,
        limit_number_opt,
    ).await;

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&cfs_session_vec).unwrap()
        );
    } else {
        common::cfs_session_utils::print_table_struct(&cfs_session_vec);
    }
}
