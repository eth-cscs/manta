use backend_dispatcher::{
    interfaces::{cfs::CfsTrait, hsm::group::GroupTrait},
    types::cfs::cfs_configuration_response::CfsConfigurationResponse,
    types::cfs::session::CfsSessionGetResponse,
};

use crate::backend_dispatcher::StaticBackendDispatcher;

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: CfsConfigurationResponse,
    pub most_recent_cfs_session_name_created: CfsSessionGetResponse,
    pub members: Vec<String>,
}

pub async fn get_details(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cluster_name: &str,
) -> Vec<ClusterDetails> {
    let mut clusters_details = vec![];

    // Get HSM groups matching cluster name
    let hsm_group = backend.get_group(shasta_token, cluster_name).await.unwrap();

    let hsm_group_name = &hsm_group.label;

    let cfs_sessions_value_vec = backend
        .get_and_filter_sessions(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(vec![hsm_group_name.to_string()]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // let most_recent_cfs_session;
    let cfs_configuration;

    for cfs_session_value in cfs_sessions_value_vec {
        // Get CFS configuration linked to CFS session related to HSM GROUP or any of its
        // members
        let configuration_name_opt = cfs_session_value
            .configuration
            .as_ref()
            .unwrap()
            .name
            .as_ref();

        let cfs_configuration_vec = backend
            .get_configuration(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                configuration_name_opt,
            )
            .await
            .unwrap();

        cfs_configuration = cfs_configuration_vec.first().unwrap();

        let cluster_details = ClusterDetails {
            hsm_group_label: hsm_group_name.to_string(),
            most_recent_cfs_configuration_name_created: cfs_configuration.clone(),
            most_recent_cfs_session_name_created: cfs_session_value,
            members: hsm_group.get_members(),
            // members: hsm::group::utils::get_member_vec_from_hsm_group(&hsm_group),
        };

        clusters_details.push(cluster_details);

        break;
    }

    clusters_details
}
