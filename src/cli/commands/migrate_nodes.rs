use std::collections::HashMap;

use hostlist_parser::parse;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    // parent_hsm_group_name: &str,
    xname_requested_hostlist: &str,
    nodryrun: bool,
    create_hsm_group: bool,
) {
    // Get list of nodes the user is targeting
    //
    // Expand hostlist to a list of xnames
    let xname_requested_vec = parse(xname_requested_hostlist)
        .expect("Error - `host_list` crate could not parse hostlist");

    println!("DEBUG - hostlist expanded: {:?}", xname_requested_vec);

    // Get final list of xnames to operate on
    // Get list of HSM groups available
    // NOTE: HSM available are the ones the user has access to
    // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

    // Get all HSM groups in the system
    // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
    // information already filtered to the client:
    // mesa::hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
    // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
    // a Vec of HsmGroups the user has access to
    let hsm_group_vec_all =
        mesa::hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .expect("Error - fetching HSM groups");

    // Create a summary of HSM groups and the list of members filtered by the list of nodes the
    // user is targeting
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();
    for hsm_group in hsm_group_vec_all {
        let hsm_group_name: String = hsm_group.label;
        let hsm_group_members: Vec<String> = hsm_group.members.unwrap().ids.unwrap();
        let xname_filtered: Vec<String> = hsm_group_members
            .iter()
            .filter(|&xname| xname_requested_vec.contains(&xname))
            .cloned()
            .collect();
        if !xname_filtered.is_empty() {
            hsm_group_summary.insert(hsm_group_name, xname_filtered);
        }
    }

    // Get list of xnames available
    let mut xname_to_move_vec: Vec<&String> = hsm_group_summary
        .iter()
        .flat_map(|(_hsm_group_name, hsm_group_members)| hsm_group_members)
        .collect();

    xname_to_move_vec.sort();
    xname_to_move_vec.dedup();

    println!(
        "DEBUG - nodes in hostlist: {}, nodes hostlist filtered by cluster/group: {}",
        xname_requested_vec.len(),
        xname_to_move_vec.len()
    );
    for (hsm_group_name, hsm_group_members) in &hsm_group_summary {
        println!("{}: {}", hsm_group_name, hsm_group_members.len());
    }
    println!("DEBUG - xnames to move: {:?}", xname_to_move_vec);

    if mesa::hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    .is_ok()
    {
        log::debug!("The HSM group {} exists, good.", target_hsm_group_name);
    } else {
        if create_hsm_group {
            log::info!("HSM group {} does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
            if nodryrun {
                mesa::hsm::group::http_client::create_new_hsm_group(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    &[],
                    "false",
                    "",
                    &[],
                )
                .await
                .expect("Unable to create new HSM group");
            } else {
                log::error!("Dry-run selected, cannot create the new group continue.");
                std::process::exit(1);
            }
        } else {
            log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
            std::process::exit(1);
        }
    }

    // Migrate nodes
    for (parent_hsm_group_name, parent_members) in hsm_group_summary {
        println!(
            "DEBUG - migrate nodes {:?} from '{}' to '{}'",
            parent_members, parent_hsm_group_name, target_hsm_group_name
        );
        let node_migration_rslt = mesa::hsm::group::utils::migrate_hsm_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_group_name,
            &parent_hsm_group_name,
            parent_members.iter().map(|xname| xname.as_str()).collect(),
            nodryrun,
        )
        .await;

        match node_migration_rslt {
            Ok((mut target_hsm_group_member_vec, mut parent_hsm_group_member_vec)) => {
                target_hsm_group_member_vec.sort();
                parent_hsm_group_member_vec.sort();
                println!(
                    "HSM '{}' members: {:?}",
                    target_hsm_group_name, target_hsm_group_member_vec
                );
                println!(
                    "HSM '{}' members: {:?}",
                    parent_hsm_group_name, parent_hsm_group_member_vec
                );
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
