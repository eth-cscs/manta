use std::collections::HashMap;

use backend_dispatcher::contracts::BackendTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::{backend_dispatcher::StaticBackendDispatcher, common};

/// Add/assign a list of xnames to a list of HSM groups
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_name: &String,
    is_regex: bool,
    xname_requested: &str,
    dryrun: bool,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
        common::node_ops::get_curated_hsm_group_from_hostregex(
            backend,
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_requested,
        )
        .await
    } else {
        // Get HashMap with HSM groups and members curated for this request.
        // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
        // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
        // hostlist have been removed
        common::node_ops::get_curated_hsm_group_from_hostlist(
            backend,
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_requested,
        )
        .await
    };

    // Get list of xnames available
    let mut xname_to_move_vec: Vec<&String> = hsm_group_summary.values().flatten().collect();

    xname_to_move_vec.sort();
    xname_to_move_vec.dedup();

    // Check if there are any xname to migrate/move and exit otherwise
    if xname_to_move_vec.is_empty() {
        println!("No hosts to move. Exit");
        std::process::exit(0);
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{:?}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
            xname_to_move_vec, target_hsm_name
        ))
        .interact()
        .unwrap()
    {
        log::info!("Continue",);
    } else {
        println!("Cancelled by user. Aborting.");
        std::process::exit(0);
    }

    let target_hsm_group = backend.get_hsm_group(shasta_token, &target_hsm_name).await;

    if target_hsm_group.is_err() {
        eprintln!(
            "Target HSM group {} does not exist, Nothing to do. Exit",
            target_hsm_name
        );
    }
    /* let target_hsm_group_vec = hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_name),
    )
    .await
    .expect("ERROR - Could not get target HSM group");

    if target_hsm_group.is_empty() {
        eprintln!(
            "Target HSM group {} does not exist, Nothing to do. Exit",
            target_hsm_name
        );
    } */

    let node_migration_rslt = backend
        .add_members_to_group(
            shasta_token,
            &target_hsm_name,
            xname_to_move_vec
                .iter()
                .map(|xname| xname.as_str())
                .collect(),
        )
        .await;
    /* let node_migration_rslt = hsm::group::utils::add_hsm_members(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &target_hsm_name,
        xname_to_move_vec
            .iter()
            .map(|xname| xname.as_str())
            .collect(),
        dryrun,
    )
    .await; */

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
