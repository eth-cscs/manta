use backend_dispatcher::interfaces::cfs::CfsTrait;

use crate::{backend_dispatcher::StaticBackendDispatcher, common};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec_opt: Option<Vec<String>>,
    xname_vec_opt: Option<Vec<&str>>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    output_opt: Option<&String>,
) {
    log::info!(
        "Get CFS sessions for HSM groups: {:?}",
        hsm_group_name_vec_opt
    );

    /* let mut cfs_session_vec = cfs::session::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        min_age_opt,
        max_age_opt,
        status_opt,
        cfs_session_name_opt,
        None,
    )
    .await
    .unwrap();

    if let Some(hsm_group_name_vec) = hsm_group_name_vec_opt {
        if !hsm_group_name_vec.is_empty() {
            cfs::session::utils::filter_by_hsm(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_session_vec,
                &hsm_group_name_vec,
                limit_number_opt,
            )
            .await;
        }
    }

    if let Some(xname_vec) = xname_vec_opt {
        cfs::session::utils::filter_by_xname(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_session_vec,
            xname_vec.as_slice(),
            limit_number_opt,
        )
        .await;
    }

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
                    .as_ref();

                let image_id = cfs_session_image_id.map(|elem| elem.as_str());

                let new_image_vec_rslt: Result<Vec<Image>, _> = ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    // hsm_group_name_vec,
                    image_id,
                )
                .await;

                // if new_image_id_vec_rslt.is_ok() && new_image_id_vec_rslt.as_ref().unwrap().first().is_some()
                if let Ok(Some(new_image)) = new_image_vec_rslt
                    .as_ref()
                    .map(|new_image_vec| new_image_vec.first())
                {
                    Some(new_image.clone().id.unwrap_or("".to_string()))
                } else {
                    None
                }
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
    } */
    let cfs_session_vec = backend
        .get_and_filter_sessions(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name_vec_opt,
            xname_vec_opt,
            min_age_opt,
            max_age_opt,
            status_opt,
            cfs_session_name_opt,
            limit_number_opt,
        )
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to get CFS sessions. Reason:\n{e}");
            std::process::exit(1);
        });

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&cfs_session_vec).unwrap()
        );
    } else {
        common::cfs_session_utils::print_table_struct(&cfs_session_vec);
    }
}
