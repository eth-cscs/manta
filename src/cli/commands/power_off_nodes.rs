use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    common::jwt_ops,
    error::Error,
    pcs::{self},
};

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_requested: &str,
    is_regex: bool,
    force: bool,
    assume_yes: bool,
    output: &str,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
    let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
        common::node_ops::get_curated_hsm_group_from_hostregex(
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
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_requested,
        )
        .await
    };

    let mut xname_vec: Vec<String> = hsm_group_summary.values().flatten().cloned().collect();

    xname_vec.sort();
    xname_vec.dedup();

    if !assume_yes {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{:?}\nThe nodes above will be powered off. Please confirm to proceed?",
                xname_vec
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

    let operation = if force { "force-off" } else { "soft-off" };

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
                "ERROR - Could not power off node/s '{:?}'. Reason:\n{}",
                xname_vec, error_msg
            );
            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power off nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
}
