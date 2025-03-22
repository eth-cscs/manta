use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{common::jwt_ops, error::Error, pcs};

use crate::{
    cli::commands::config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all,
    common::{self, audit::Audit, kafka::Kafka},
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hosts_string: &str,
    is_regex: bool,
    force: bool,
    assume_yes: bool,
    output: &str,
    kafka_audit: &Kafka,
) {
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
    /* let _ = mesa::hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
    .await; */

    // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
    let mut xname_vec = if crate::cli::commands::power_on_nodes::is_user_input_nids(hosts_string) {
        log::debug!("User input seems to be NID");
        common::node_ops::nid_to_xname(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            hosts_string,
            is_regex,
        )
        .await
        .expect("Could not convert NID to XNAME")
    } else {
        log::debug!("User input seems to be XNAME");
        let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
            common::node_ops::get_curated_hsm_group_from_xname_regex(
                &hosts_string,
                hsm_group_available_map.clone(),
                false,
            )
            .await
        } else {
            // Get HashMap with HSM groups and members curated for this request.
            // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
            // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
            // hostlist have been removed
            common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                &hosts_string,
                hsm_group_available_map.clone(),
                false,
            )
            .await
        };

        hsm_group_summary.values().flatten().cloned().collect()
    };

    if xname_vec.is_empty() {
        eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
        std::process::exit(0);
    }

    xname_vec.sort();
    xname_vec.dedup();

    if !assume_yes {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{:?}\nThe nodes above will restart. Please confirm to proceed?",
                xname_vec.join(", ")
            ))
            .interact()
            .unwrap()
        {
            log::info!("Continue",);
        } else {
            println!("Cancelled by user. Aborting.");
            std::process::exit(0);
        }
    }

    // Restart node
    let operation = if force {
        "hard-restart"
    } else {
        "soft-restart"
    };

    let power_mgmt_summary_rslt = pcs::transitions::http_client::post_block(
        shasta_base_url,
        shasta_token,
        shasta_root_cert,
        operation,
        &xname_vec,
    )
    .await;

    let power_mgmt_summary = match power_mgmt_summary_rslt {
        Ok(value) => value,
        Err(e) => {
            /* eprintln!(
                "ERROR - Could not restart node/s '{:?}'. Reason:\n{}",
                xname_vec, error_msg
            );
            std::process::exit(1); */
            let error_msg = match e {
                Error::CsmError(value) => serde_json::to_string_pretty(&value).unwrap(),
                Error::SerdeError(value) => value.to_string(),
                Error::IoError(value) => value.to_string(),
                Error::NetError(value) => value.to_string(),
                Error::Message(value) => value.to_string(),
            };
            eprintln!(
                "ERROR - Could not restart node/s '{:?}'. Reason:\n{}",
                xname_vec, error_msg
            );
            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let group_vec = mesa::hsm::group::utils::get_hsm_group_vec_from_xname_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec
            .iter()
            .map(|xname| xname.as_str())
            .collect::<Vec<&str>>(),
    )
    .await;

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "group": group_vec, "message": "power reset"});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
    // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power reset nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
}
