use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::common::jwt_ops;

use crate::common::{self, audit::Audit, kafka::Kafka};

use super::config_show::get_hsm_name_available_from_jwt_or_all;

/// Add/assign a list of xnames to a list of HSM groups
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_name: &String,
    is_regex: bool,
    xname_requested: &str,
    dryrun: bool,
    kafka_audit: &Kafka,
) {
    let hsm_name_available_vec =
        get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    // Get HSM group user has access to
    let hsm_group_available_map = mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
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
    let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
        common::node_ops::get_curated_hsm_group_from_xname_regex(
            xname_requested,
            hsm_group_available_map,
            false,
        )
        .await
    } else {
        // Get HashMap with HSM groups and members curated for this request.
        // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
        // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
        // hostlist have been removed
        common::node_ops::get_curated_hsm_group_from_xname_hostlist(
            xname_requested,
            hsm_group_available_map,
            false,
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

    let target_hsm_group_vec = mesa::hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_name),
    )
    .await
    .expect("ERROR - Could not get target HSM group");

    if target_hsm_group_vec.is_empty() {
        eprintln!(
            "Target HSM group {} does not exist, Nothing to do. Exit",
            target_hsm_name
        );
    }

    let node_migration_rslt = mesa::hsm::group::utils::add_hsm_members(
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
    }

    // Audit
    let msg_data = format!(
        "User: {} ({}) ; Operation: Add nodes {:?} to group '{}'",
        jwt_ops::get_name(shasta_token).unwrap(),
        jwt_ops::get_preferred_username(shasta_token).unwrap(),
        xname_to_move_vec,
        target_hsm_name
    );

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
