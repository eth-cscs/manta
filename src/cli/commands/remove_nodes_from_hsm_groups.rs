use std::collections::HashMap;

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_name_vec: Vec<String>,
    xname_requested_hostlist: &str,
    nodryrun: bool,
    remove_empty_hsm_group: bool,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    // Get HashMap with HSM groups and members curated for this request.
    // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
    // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
    // hostlist have been removed
    let mut hsm_group_summary: HashMap<String, Vec<String>> =
        crate::common::node_ops::get_curated_hsm_group_from_hostlist(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_requested_hostlist,
        )
        .await;

    // Keep HSM groups based on list of target HSM groups provided
    hsm_group_summary.retain(|hsm_name, _xname_vec| target_hsm_name_vec.contains(hsm_name));

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

    for target_hsm_name in target_hsm_name_vec {
        if mesa::hsm::group::http_client::get(
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
            if remove_empty_hsm_group {
                log::info!(
                    "HSM group {} does not exist, it will be created",
                    target_hsm_name
                );
            } else {
                log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_name);
                std::process::exit(1);
            }
        }

        for (target_hsm_name, xname_to_move_vec) in &hsm_group_summary {
            let node_migration_rslt = mesa::hsm::group::utils::remove_hsm_members(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &target_hsm_name,
                xname_to_move_vec
                    .iter()
                    .map(|xname| xname.as_str())
                    .collect(),
                nodryrun,
            )
            .await;

            match node_migration_rslt {
                Ok(mut target_hsm_group_member_vec) => {
                    target_hsm_group_member_vec.sort();
                    println!(
                        "HSM '{}' members: {:?}",
                        target_hsm_name, target_hsm_group_member_vec
                    );
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}
