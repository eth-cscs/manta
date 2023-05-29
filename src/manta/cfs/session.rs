use crate::shasta;

pub async fn get_sessions(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    session_name: Option<&String>,
    limit_number: Option<&u8>,
) -> Vec<Vec<String>> {
    let cfs_sessions = shasta::cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        session_name,
        limit_number,
        None,
    )
    .await
    .unwrap_or_default();

    log::info!("CFS sessions:\n{:#?}", cfs_sessions);

    let bos_sessiontemplate_list =
        shasta::bos::template::http_client::get(shasta_token, shasta_base_url, None, None, None)
            .await
            .unwrap();

    let mut cfs_session_table_data_list = Vec::new();

    for cfs_session in cfs_sessions {
        let mut cfs_session_table_data = Vec::new();
        let cfs_sesion_name = cfs_session["name"].as_str().unwrap();
        cfs_session_table_data.push(cfs_sesion_name.to_owned());
        let cfs_session_configuration_name = cfs_session["configuration"]["name"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_configuration_name.to_owned());
        let cfs_session_target_definition = cfs_session["target"]["definition"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_target_definition.to_owned());
        let cfs_session_target_groups = if cfs_session["target"]["groups"].as_array().is_some()
            && (cfs_session["target"]["groups"]
                .as_array()
                .unwrap()
                .iter()
                .len()
                > 0)
        {
            let cfs_session_target_groups_json =
                cfs_session["target"]["groups"].as_array().unwrap();

            let mut cfs_session_target_groups_aux =
                String::from(cfs_session_target_groups_json[0]["name"].as_str().unwrap());

            for (i, _) in cfs_session_target_groups_json.iter().enumerate().skip(1) {
                if i % 2 == 0 {
                    // breaking the cell content into multiple lines (only 2 target groups per line)
                    cfs_session_target_groups_aux.push_str(",\n");
                    // target_groups = format!("{},\n", target_groups);
                } else {
                    cfs_session_target_groups_aux.push_str(", ");
                    // target_groups = format!("{}, ", target_groups);
                }

                cfs_session_target_groups_aux
                    .push_str(cfs_session_target_groups_json[i]["name"].as_str().unwrap());
            }

            cfs_session_target_groups_aux
        } else {
            "".to_string()
        };

        let mut cfs_session_ansible_limit = cfs_session["ansible"]["limit"]
            .as_str()
            .unwrap_or_default()
            .split(',')
            .map(|xname| xname.trim());

        let first = cfs_session_ansible_limit.next();

        let cfs_session_ansible_limit = if let Some(first_xname) = first {
            let mut cfs_session_ansible_limit_aux = String::from(first_xname);

            let mut i = 1;

            for cfs_session_ansible_limit in cfs_session_ansible_limit {
                if i % 2 == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)
                    cfs_session_ansible_limit_aux.push_str(", \n");
                    // ansible_limits = format!("{},\n", ansible_limits);
                } else {
                    cfs_session_ansible_limit_aux.push_str(", ");
                    // ansible_limits = format!("{}, ", ansible_limits);
                }

                cfs_session_ansible_limit_aux.push_str(cfs_session_ansible_limit);
                // ansible_limits = format!("{}{}", ansible_limits, ansible_limit);

                i += 1;
            }

            cfs_session_ansible_limit_aux
        } else {
            "".to_string()
        };

        let cfs_session_target = if !cfs_session_target_groups.is_empty() {
            &cfs_session_target_groups
        } else {
            &cfs_session_ansible_limit
        };
        cfs_session_table_data.push(cfs_session_target.to_string());
        let cfs_session_status_session_starttime = cfs_session["status"]["session"]["startTime"]
            .as_str()
            .unwrap();
        cfs_session_table_data.push(cfs_session_status_session_starttime.to_string());

        let cfs_session_status_session_status =
            cfs_session["status"]["session"]["status"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_status_session_status.to_string());

        let cfs_session_status_session_succeeded = cfs_session["status"]["session"]["succeeded"]
            .as_str()
            .unwrap();
        cfs_session_table_data.push(cfs_session_status_session_succeeded.to_string());

        let cfs_session_status_artifacts_result_id = if !cfs_session["status"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            cfs_session["status"]["artifacts"][0]["result_id"]
                .as_str()
                .unwrap()
        } else {
            ""
        };

        // println!("{:#?}", cfs_session);

        let mut image_id_from_bos_sessiontemplate = "";
        if !cfs_session_status_artifacts_result_id.is_empty() {
            let bos_sessiontemplate =
                bos_sessiontemplate_list
                    .iter()
                    .find(|bos_session_template| {
                        bos_session_template
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .eq(cfs_session_configuration_name)
                    });
            if bos_sessiontemplate.is_some() {
                for (_boot_sets_param, boot_sets_value) in bos_sessiontemplate.unwrap()["boot_sets"]
                    .as_object()
                    .unwrap()
                {
                    if boot_sets_value.get("path").is_some() {
                        image_id_from_bos_sessiontemplate = boot_sets_value["path"]
                            .as_str()
                            .unwrap()
                            .trim_start_matches("s3://boot-images/")
                            .trim_end_matches("/manifest.json");
                        break;
                    }
                }
            } else {
                image_id_from_bos_sessiontemplate = cfs_session_status_artifacts_result_id;
            }
        }
        cfs_session_table_data.push(image_id_from_bos_sessiontemplate.to_string());
        // println!("{:#?}", ims_image_kernel_path);
        // println!("Hey! {:?}", cfs_session_table_data);
        cfs_session_table_data_list.push(cfs_session_table_data);
    }

    cfs_session_table_data_list
}
