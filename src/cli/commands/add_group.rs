use std::collections::HashMap;

use dialoguer::theme::ColorfulTheme;
use mesa::{common::jwt_ops, hsm::group::r#struct::HsmGroup};

use crate::{
    cli::process::validate_target_hsm_members,
    common::{self, audit::Audit, kafka::Kafka},
};

use super::{
    config_show::get_hsm_name_available_from_jwt_or_all, power_on_nodes::is_user_input_nids,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    label: &str,
    hosts_string_opt: Option<&String>,
    assume_yes: bool,
    is_regex: bool,
    dryrun: bool,
    kafka_audit: &Kafka,
) {
    let xname_vec_opt = if let Some(hosts_string) = hosts_string_opt {
        let hsm_name_available_vec =
            get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await;

        // Get HSM group user has access to
        let hsm_group_available_map =
            mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
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
        // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
        let mut xname_vec = if is_user_input_nids(hosts_string) {
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
                    &hosts_string,
                    hsm_group_available_map,
                    false,
                )
                .await
            };

            hsm_group_summary.values().flatten().cloned().collect()
        };

        xname_vec.sort();
        xname_vec.dedup();

        let xname_vec_opt = if xname_vec.is_empty() {
            None
        } else {
            Some(xname_vec)
        };

        xname_vec_opt
    } else {
        None
    };

    // Validate user has access to the list of xnames requested
    if let Some(xname_vec) = &xname_vec_opt {
        validate_target_hsm_members(
            &shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_vec.iter().map(|xname| xname.to_string()).collect(),
        )
        .await;
    }

    // Create Group instance for http payload
    let group = HsmGroup::new(label, xname_vec_opt.clone(), None, None);

    if !assume_yes {
        let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will create the group below:\n{}\nPlease confirm to proceed",
                serde_json::to_string_pretty(&group).unwrap()
            ))
            .interact()
            .unwrap();

        if !proceed {
            println!("Operation canceled by the user. Exit");
            std::process::exit(1);
        }
    }

    if dryrun {
        println!(
            "Dryrun mode: The group below would be created:\n{}",
            serde_json::to_string_pretty(&group).unwrap()
        );
        return;
    }

    // Call backend to create group
    let group_rslt = mesa::hsm::group::http_client::post(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        group.into(),
    )
    .await;

    match group_rslt {
        Ok(_) => {
            println!("Group '{}' created", label);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec_opt.unwrap_or_default()}, "message": format!("Create Group '{}'", label)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
