use crate::common::{cluster_ops, node_ops};

pub async fn exec(
    // hsm_group: Option<&String>,
    // cli_get_hsm_groups: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: &str,
) {
    /* let hsm_group_name = match hsm_group {
        None => cli_get_hsm_groups.get_one::<String>("HSMGROUP").unwrap(),
        Some(hsm_group_name_value) => hsm_group_name_value,
    }; */

    let hsm_groups = cluster_ops::get_details(shasta_token, shasta_base_url, hsm_group_name).await;

    for hsm_group in hsm_groups {
        println!("************************* HSM GROUP *************************");

        println!(" * HSM group label: {}", hsm_group.hsm_group_label);
        println!(" * CFS configuration details:");
        println!(
            "   - name: {}",
            hsm_group.most_recent_cfs_configuration_name_created["name"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "   - last updated: {}",
            hsm_group.most_recent_cfs_configuration_name_created["lastUpdated"]
                .as_str()
                .unwrap_or_default()
        );

        for (i, layer) in hsm_group.most_recent_cfs_configuration_name_created["layers"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .enumerate()
        {
            println!("   + Layer {}", i);
            println!(
                "     - name: {}",
                layer["name"].as_str().unwrap_or_default()
            );
            println!(
                "     - url: {}",
                layer["cloneUrl"].as_str().unwrap_or_default()
            );
            println!(
                "     - commit: {}",
                layer["commit"].as_str().unwrap_or_default()
            );
            println!(
                "     - playbook: {}",
                layer["playbook"].as_str().unwrap_or_default()
            );
        }

        println!(" * CFS session details:");
        println!(
            "   - Name: {}",
            hsm_group.most_recent_cfs_session_name_created["name"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "   - Configuration name: {}",
            hsm_group.most_recent_cfs_session_name_created["configuration"]["name"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "   - Target: {}",
            hsm_group.most_recent_cfs_session_name_created["target"]["definition"]
                .as_str()
                .unwrap_or_default()
        );
        println!("   + Ansible details:");
        println!(
            "     - name: {}",
            hsm_group.most_recent_cfs_session_name_created["ansible"]["config"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "     - limit: {}",
            hsm_group.most_recent_cfs_session_name_created["ansible"]["limit"]
                .as_str()
                .unwrap_or_default()
        );
        println!("   + Status:");
        println!(
            "     - status: {}",
            hsm_group.most_recent_cfs_session_name_created["status"]["session"]["status"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "     - succeeded: {}",
            hsm_group.most_recent_cfs_session_name_created["status"]["session"]["succeeded"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "     - job: {}",
            hsm_group.most_recent_cfs_session_name_created["status"]["session"]["job"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "     - start: {}",
            hsm_group.most_recent_cfs_session_name_created["status"]["session"]["startTime"]
                .as_str()
                .unwrap_or_default()
        );
        println!(
            "   - tags: {}",
            hsm_group.most_recent_cfs_session_name_created["tags"]
        );

        println!(
            " * members: {}",
            node_ops::nodes_to_string_format_one_line(Some(&hsm_group.members))
        );
    }
}
