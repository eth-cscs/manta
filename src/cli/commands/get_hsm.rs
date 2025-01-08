use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{cluster_ops, node_ops},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: &str,
) {
    let hsm_groups = cluster_ops::get_details(
        &backend,
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name,
    )
    .await;

    for hsm_group in hsm_groups {
        println!("************************* HSM GROUP *************************");

        println!(" * HSM group label: {}", hsm_group.hsm_group_label);
        println!(" * CFS configuration details:");
        println!(
            "   - name: {}",
            hsm_group.most_recent_cfs_configuration_name_created.name
        );
        println!(
            "   - last updated: {}",
            hsm_group
                .most_recent_cfs_configuration_name_created
                .last_updated
        );

        for (i, layer) in hsm_group
            .most_recent_cfs_configuration_name_created
            .layers
            .iter()
            .enumerate()
        {
            println!("   + Layer {}", i);
            println!("     - name: {}", layer.name);
            println!("     - url: {}", layer.clone_url);
            println!("     - commit: {}", layer.commit.as_ref().unwrap());
            println!("     - playbook: {}", layer.playbook);
        }

        println!(" * CFS session details:");
        println!(
            "   - Name: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .name
                .unwrap_or_default()
        );
        println!(
            "   - Configuration name: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .configuration
                .as_ref()
                .unwrap()
                .name
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "   - Target: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .target
                .as_ref()
                .unwrap()
                .definition
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!("   + Ansible details:");
        println!(
            "     - name: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .ansible
                .as_ref()
                .unwrap()
                .config
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "     - limit: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .ansible
                .as_ref()
                .unwrap()
                .limit
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!("   + Status:");
        println!(
            "     - status: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .status
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "     - succeeded: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .succeeded
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "     - job: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .job
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "     - start: {}",
            hsm_group
                .most_recent_cfs_session_name_created
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .start_time
                .as_ref()
                .cloned()
                .unwrap_or_default()
        );
        println!(
            "   - tags: {:#?}",
            hsm_group
                .most_recent_cfs_session_name_created
                .tags
                .as_ref()
                .unwrap()
        );

        println!(
            " * members: {}",
            node_ops::nodes_to_string_format_one_line(Some(&hsm_group.members))
        );
    }
}
