use serde_json::Value;

use crate::shasta::{cfs::configuration, hsm::http_client::get_hsm_groups};

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: Value,
    pub most_recent_cfs_session_name_created: Value,
    pub members: Vec<Value>,
}

pub async fn get_details(
    shasta_token: &str,
    shasta_base_url: &str,
    cluster_name: &str,
) -> Vec<ClusterDetails> {
    let mut clusters_details = vec![];

    // Get HSM groups matching cluster name
    let hsm_groups = get_hsm_groups(
        shasta_token,
        shasta_base_url,
        Some(&cluster_name.to_string()),
    )
    .await
    .unwrap();

    for hsm_group in hsm_groups {
        let hsm_group_name = hsm_group["label"].as_str().unwrap();

        let hsm_group_members: String =
            crate::shasta::hsm::utils::get_members_from_hsm_group_serde_value(&hsm_group).join(",");

        // Get all CFS sessions
        let cfs_sessions_value_vec = crate::shasta::cfs::session::http_client::get(
            shasta_token,
            shasta_base_url,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let most_recent_cfs_session;
        let cfs_configuration;

        for cfs_session_value in cfs_sessions_value_vec {
            // println!("cfs_session_value:\n{:#?}", cfs_session_value);
            let target_groups_option = cfs_session_value.pointer("/target/groups");
            let target_groups = if Some(Value::Null) == target_groups_option.cloned()
                || target_groups_option.is_none()
            {
                Vec::new()
            } else {
                target_groups_option
                    .unwrap()
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
            };
            let ansible_limit_option = cfs_session_value.pointer("/ansible/limit");
            let ansible_limit = if Some(Value::Null) == ansible_limit_option.cloned()
                || ansible_limit_option.is_none()
            {
                ""
            } else {
                ansible_limit_option.unwrap().as_str().unwrap()
            };

            // Check CFS session is linkged to HSM GROUP name or any of its members
            if target_groups
                .iter()
                .map(|target_group| target_group["name"].as_str().unwrap())
                .collect::<Vec<&str>>()
                .contains(&hsm_group_name)
                || ansible_limit.contains(&hsm_group_members)
            {
                most_recent_cfs_session = cfs_session_value;

                // Get CFS configuration linked to CFS session related to HSM GROUP or any of its
                // members
                let cfs_configuration_vec = configuration::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    Some(
                        &most_recent_cfs_session
                            .pointer("/configuration/name")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                    ),
                    None,
                )
                .await
                .unwrap();

                cfs_configuration = cfs_configuration_vec.first().unwrap();

                let cluster_details = ClusterDetails {
                    hsm_group_label: hsm_group_name.to_string(),
                    most_recent_cfs_configuration_name_created: cfs_configuration.clone(),
                    most_recent_cfs_session_name_created: most_recent_cfs_session,
                    members: hsm_group["members"]["ids"].as_array().unwrap().clone(),
                };

                clusters_details.push(cluster_details);

                break;
            }
        }
    }

    clusters_details
}
