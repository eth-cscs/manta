use std::collections::HashMap;

use backend_dispatcher::interfaces::group::GroupTrait;

use crate::backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    target_hsm_name_vec: Vec<String>,
    parent_hsm_name_vec: Vec<String>,
    xname_requested_hostlist: &str,
    nodryrun: bool,
    create_hsm_group: bool,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    // Get HashMap with HSM groups and members curated for this request.
    // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
    // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
    // hostlist have been removed
    let mut hsm_group_summary: HashMap<String, Vec<String>> =
        crate::common::node_ops::get_curated_hsm_group_from_xname_hostlist(
            backend,
            shasta_token,
            xname_requested_hostlist,
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

    for target_hsm_name in target_hsm_name_vec {
        if backend
            .get_group(shasta_token, &target_hsm_name)
            .await
            .is_ok()
        /* if hsm::group::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&target_hsm_name),
        )
        .await
        .is_ok() */
        {
            log::debug!("The HSM group {} exists, good.", target_hsm_name);
        } else {
            if create_hsm_group {
                log::info!(
                    "HSM group {} does not exist, it will be created",
                    target_hsm_name
                );
                if nodryrun {
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
            let node_migration_rslt = backend
                .migrate_group_members(
                    shasta_token,
                    &target_hsm_name,
                    &parent_hsm_name,
                    xname_to_move_vec
                        .iter()
                        .map(|xname| xname.as_str())
                        .collect(),
                )
                .await;
            /* let node_migration_rslt = hsm::group::utils::migrate_hsm_members(
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
            .await; */

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
}
