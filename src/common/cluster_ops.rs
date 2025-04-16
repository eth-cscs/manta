use mesa::{
    cfs::{
        configuration::mesa::r#struct::cfs_configuration_response::v2::CfsConfigurationResponse,
        session::mesa::r#struct::v2::CfsSessionGetResponse,
    },
    hsm,
};

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: CfsConfigurationResponse,
    pub most_recent_cfs_session_name_created: CfsSessionGetResponse,
    pub members: Vec<String>,
}

pub async fn get_details(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cluster_name: &str,
) -> Vec<ClusterDetails> {
    let mut clusters_details = vec![];

    // Get HSM groups matching cluster name
    let hsm_groups = hsm::group::http_client::get_hsm_group_without_system_wide_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&cluster_name.to_string()),
    )
    .await
    .unwrap();

    for hsm_group in &hsm_groups {
        let hsm_group_name = &hsm_group.label;

        /* let hsm_group_members: String =
        hsm::group::shasta::utils::get_member_vec_from_hsm_group_value(&hsm_group).join(","); */

        // Get all CFS sessions
        let mut cfs_sessions_value_vec = mesa::cfs::session::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
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
            true,
        )
        .await;

        // let most_recent_cfs_session;
        let cfs_configuration;

        for cfs_session_value in cfs_sessions_value_vec {
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
                members: hsm::group::utils::get_member_vec_from_hsm_group(hsm_group),
            };

            clusters_details.push(cluster_details);

            break;
        }
    }

    clusters_details
}
