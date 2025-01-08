use std::collections::HashMap;

use backend_dispatcher::contracts::BackendTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::backend_dispatcher::StaticBackendDispatcher;

/// Remove/unassign a list of xnames to a list of HSM groups
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
        crate::common::node_ops::get_curated_hsm_group_from_hostregex(
            &backend,
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
        crate::common::node_ops::get_curated_hsm_group_from_hostlist(
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
            "{:?}\nThe nodes above will be removed from HSM group '{}'. Do you want to proceed?",
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

    if backend
        .get_group(shasta_token, target_hsm_name)
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
    }

    if dryrun {
        println!(
            "dryrun - Delete nodes {:?} in {}",
            xname_to_move_vec, target_hsm_name
        );
        std::process::exit(0);
    }

    // Remove xnames from HSM group
    for xname in xname_to_move_vec {
        let _ = backend
            .delete_member_from_group(shasta_token, &target_hsm_name, xname)
            .await
            .unwrap();
    }
    /* let node_migration_rslt = hsm::group::utils::remove_hsm_members(
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
    } */
}
