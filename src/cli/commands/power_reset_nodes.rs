use backend_dispatcher::interfaces::pcs::PCSTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::common::jwt_ops;

use crate::{backend_dispatcher::StaticBackendDispatcher, common};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    hosts_string: &str,
    is_regex: bool,
    force: bool,
    assume_yes: bool,
    output: &str,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    /* let _ = mesa::hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
    .await; */

    // Convert user input to xname
    let mut xname_vec = common::node_ops::resolve_node_list_user_input_to_xname(
        backend,
        shasta_token,
        hosts_string,
        false,
        is_regex,
    )
    .await
    .unwrap_or_else(|e| {
        eprintln!(
            "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
            e
        );
        std::process::exit(1);
    });

    /* // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
    let mut xname_vec = if is_user_input_nids(hosts_string) {
        log::debug!("User input seems to be NID");
        backend
            .nid_to_xname(shasta_token, hosts_string, is_regex)
            .await
            .expect("Could not convert NID to XNAME")
        /* common::node_ops::nid_to_xname(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            hosts_string,
            is_regex,
        )
        .await
        .expect("Could not convert NID to XNAME") */
    } else {
        log::debug!("User input seems to be XNAME");
        let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
            common::node_ops::get_curated_hsm_group_from_xname_regex(
                backend,
                shasta_token,
                /* shasta_base_url,
                shasta_root_cert, */
                &hosts_string,
            )
            .await
        } else {
            // Get HashMap with HSM groups and members curated for this request.
            // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
            // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
            // hostlist have been removed
            common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                backend,
                shasta_token,
                &hosts_string,
            )
            .await
        };

        hsm_group_summary.values().flatten().cloned().collect()
    }; */

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

    /* let operation = if force {
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
    .await
    .map_err(|e| Error::Message(e.to_string())); */
    let power_mgmt_summary_rslt = backend
        .power_reset_sync(shasta_token, &xname_vec, force)
        .await;

    let power_mgmt_summary = match power_mgmt_summary_rslt {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "ERROR - Could not restart node/s '{:?}'. Reason:\n{}",
                xname_vec,
                e.to_string()
            );

            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power reset nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
}
