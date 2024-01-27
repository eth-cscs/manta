use mesa::{
    cfs::{
        configuration::mesa::r#struct::cfs_configuration_response::CfsConfigurationResponse,
        session::mesa::r#struct::CfsSessionGetResponse,
    },
    hsm,
};
use serde_json::Value;

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: CfsConfigurationResponse,
    pub most_recent_cfs_session_name_created: CfsSessionGetResponse,
    pub members: Vec<Value>,
}

pub async fn get_details(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cluster_name: &str,
) -> Vec<ClusterDetails> {
    let mut clusters_details = vec![];

    // Get HSM groups matching cluster name
    let hsm_groups = hsm::group::shasta::http_client::get_hsm_group_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&cluster_name.to_string()),
    )
    .await
    .unwrap();

    for hsm_group in hsm_groups {
        let hsm_group_name = hsm_group["label"].as_str().unwrap();

        /* let hsm_group_members: String =
        hsm::group::shasta::utils::get_member_vec_from_hsm_group_value(&hsm_group).join(","); */

        // Get all CFS sessions
        let mut cfs_sessions_value_vec = mesa::cfs::session::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
        )
        .await
        .unwrap();

        mesa::cfs::session::mesa::utils::filter_by_hsm(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_sessions_value_vec,
            &[hsm_group_name.to_string()],
            None,
        )
        .await;

        // let most_recent_cfs_session;
        let cfs_configuration;

        for cfs_session_value in cfs_sessions_value_vec {
            /* let target_groups = cfs_session_value
            .target
            .as_ref()
            .unwrap()
            .groups
            .as_ref()
            .map(|target_groups| {
                if target_groups.is_empty() {
                    Vec::new()
                } else {
                    target_groups.to_vec()
                }
            })
            .unwrap(); */

            /* let ansible_limit = cfs_session_value
            .ansible
            .as_ref()
            .unwrap()
            .limit
            .as_ref()
            .map(|ansible_limit| {
                if ansible_limit.is_empty() {
                    ""
                } else {
                    ansible_limit
                }
            })
            .unwrap(); */

            /* // Check CFS session is linkged to HSM GROUP name or any of its members
            if target_groups
                .iter()
                .map(|target_group| target_group.name.as_str())
                .collect::<Vec<&str>>()
                .contains(&hsm_group_name)
                || ansible_limit.contains(&hsm_group_members)
            { */

            // Get CFS configuration linked to CFS session related to HSM GROUP or any of its
            // members
            let cfs_configuration_vec = mesa::cfs::configuration::mesa::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(
                    cfs_session_value
                        .configuration
                        .as_ref()
                        .unwrap()
                        .name
                        .as_ref()
                        .unwrap(),
                ),
            )
            .await
            .unwrap();

            cfs_configuration = cfs_configuration_vec.first().unwrap();

            let cluster_details = ClusterDetails {
                hsm_group_label: hsm_group_name.to_string(),
                most_recent_cfs_configuration_name_created: cfs_configuration.clone(),
                most_recent_cfs_session_name_created: cfs_session_value,
                members: hsm_group["members"]["ids"].as_array().unwrap().clone(),
            };

            clusters_details.push(cluster_details);

            break;
            // }
        }
    }

    clusters_details
}
