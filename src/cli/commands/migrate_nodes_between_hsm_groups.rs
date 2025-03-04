use std::collections::HashMap;

use mesa::common::jwt_ops;

use crate::{
    cli::commands::config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all,
    common::{audit::Audit, kafka::Kafka},
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_name_vec: Vec<String>,
    parent_hsm_name_vec: Vec<String>,
    xname_requested_hostlist: &str,
    nodryrun: bool,
    create_hsm_group: bool,
    kafka_audit: &Kafka,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    // Get HashMap with HSM groups and members curated for this request.
    // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
    // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
    // hostlist have been removed
    let hsm_name_available_vec = get_hsm_name_without_system_wide_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    // Get HSM group user has access to
    let hsm_group_available_map =
        mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_without_system_wide_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Filter xnames to the ones members to HSM groups the user has access to
    //
    let mut hsm_group_summary: HashMap<String, Vec<String>> =
        crate::common::node_ops::get_curated_hsm_group_from_xname_hostlist(
            xname_requested_hostlist,
            hsm_group_available_map,
            false,
        )
        .await;

    // Keep HSM groups based on list of parent HSM groups provided
    hsm_group_summary.retain(|hsm_name, _xname_vec| parent_hsm_name_vec.contains(hsm_name));

    // Get list of xnames available
    let mut xname_to_move_vec: Vec<&String> = hsm_group_summary
        .iter()
        .flat_map(|(_hsm_group_name, hsm_group_members)| hsm_group_members)
        .collect();

    xname_to_move_vec.sort();
    xname_to_move_vec.dedup();

    // Check if there are any xname to migrate/move and exit otherwise
    if xname_to_move_vec.is_empty() {
        println!("No hosts to move. Exit");
        std::process::exit(0);
    }

    log::info!("List of all nodes to work on per HSM group:");
    for (hsm_group_name, hsm_group_members) in &hsm_group_summary {
        log::info!("{}: {}", hsm_group_name, hsm_group_members.len());
    }
    log::debug!("xnames to move: {:?}", xname_to_move_vec);

    for target_hsm_name in &target_hsm_name_vec {
        if mesa::hsm::group::http_client::get_without_system_wide(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&target_hsm_name),
        )
        .await
        .is_ok()
        {
            log::debug!("The HSM group {} exists, good.", target_hsm_name);
        } else {
            if create_hsm_group {
                log::info!(
                    "HSM group {} does not exist, it will be created",
                    target_hsm_name
                );
                if nodryrun {
                    /* mesa::hsm::group::http_client::create_new_hsm_group(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        &target_hsm_name,
                        &[],
                        "false",
                        "",
                        &[],
                    )
                    .await
                    .expect("Unable to create new HSM group"); */
                } else {
                    log::error!("Dry-run selected, cannot create the new group continue.");
                    std::process::exit(1);
                }
            } else {
                log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_name);
                std::process::exit(1);
            }
        }

        // Migrate nodes
        for (parent_hsm_name, xname_to_move_vec) in &hsm_group_summary {
            let node_migration_rslt = mesa::hsm::group::utils::migrate_hsm_members(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &target_hsm_name,
                &parent_hsm_name,
                xname_to_move_vec
                    .iter()
                    .map(|xname| xname.as_str())
                    .collect(),
                nodryrun,
            )
            .await;

            match node_migration_rslt {
                Ok((mut target_hsm_group_member_vec, mut parent_hsm_group_member_vec)) => {
                    target_hsm_group_member_vec.sort();
                    parent_hsm_group_member_vec.sort();
                    println!(
                        "HSM '{}' members: {:?}",
                        target_hsm_name, target_hsm_group_member_vec
                    );
                    println!(
                        "HSM '{}' members: {:?}",
                        parent_hsm_name, parent_hsm_group_member_vec
                    );
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_to_move_vec}, "message": format!("Migrate nodes from {:?} to {:?}", parent_hsm_name_vec, target_hsm_name_vec)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
