use serde_json::Value;

use crate::shasta::{hsm::http_client::get_hsm_groups, cfs::{configuration, session}};

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: Value,
    pub most_recent_cfs_session_name_created: Value,
    pub members: Vec<Value>
}

pub async fn get_details(shasta_token: &str, shasta_base_url: &str, cluster_name: &str) -> Vec<ClusterDetails> {
    
    let mut clusters_details = vec!();

    // Get HSM groups matching cluster name
    let hsm_groups = get_hsm_groups(shasta_token, shasta_base_url, Some(&cluster_name.to_string())).await.unwrap();

    for hsm_group in hsm_groups {

        // Get most recent CFS configuration
        let mut cfs_configurations= configuration::http_client::get(shasta_token, shasta_base_url, Some(&cluster_name.to_string()), None, Some(&1)).await.unwrap_or_else(|_| vec!());

        let most_recept_cfs_configuration_created;

        if !cfs_configurations.is_empty() {
            most_recept_cfs_configuration_created = cfs_configurations.swap_remove(0)
        } else {
            most_recept_cfs_configuration_created = Value::Null;
        }

        // Get most recent CFS session
        let mut cfs_sessions = session::http_client::get(shasta_token, shasta_base_url, Some(&cluster_name.to_string()), None, Some(&1)).await.unwrap_or_else(|_| vec!());

        let most_recept_cfs_session_created;

        if !cfs_sessions.is_empty() {
            most_recept_cfs_session_created = cfs_sessions.swap_remove(0)
        } else {
            most_recept_cfs_session_created = Value::Null;
        }

        let cluster_details = ClusterDetails {
            hsm_group_label: hsm_group["label"].as_str().unwrap_or_default().to_string(),
            most_recent_cfs_configuration_name_created: most_recept_cfs_configuration_created,
            most_recent_cfs_session_name_created: most_recept_cfs_session_created,
            members: hsm_group["members"]["ids"].as_array().unwrap().clone(),
        };

        clusters_details.push(cluster_details);
    }

    clusters_details

}