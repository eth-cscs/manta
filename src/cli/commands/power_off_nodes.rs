use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{backend::Csm, common::jwt_ops, error::Error, pcs};

use crate::{backend::StaticBackendDispatcher, common};

pub async fn exec(
    backend: StaticBackendDispatcher,
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

    /* let backup_config_rslt: impl backend_dispatcher::contracts::Boot
        + backend_dispatcher::contracts::Power
        + backend_dispatcher::contracts::Authentication = if infra_backend == "csm" {
        mesa::backend::Config::new(
            &shasta_base_url.to_string(),
            Some(&shasta_token.to_string()),
            &shasta_root_cert.to_vec(),
            "site_name", // FIXME: do not hardcode this value and move it to config file
        )
        .await
    } else if infra_backend == "ochami" {
        silla::backend::Config::new(
            &shasta_base_url,
            Some(shasta_token),
            shasta_root_cert,
            "site_name", // FIXME: do not hardcode this value and move it to config file
        )
        .await
    } else {
        eprintln!("ERROR - Infra backend not supported. Exit");
        std::process::exit(1);
    };

    let backup_config = match backup_config_rslt {
        Ok(backup_config) => backup_config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }; */

    let operation = if force { "force-off" } else { "soft-off" };

    let power_mgmt_summary_rslt = pcs::transitions::http_client::post_block(
        shasta_base_url,
        shasta_token,
        shasta_root_cert,
        operation,
        &xname_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()));

    let power_mgmt_summary = match power_mgmt_summary_rslt {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "ERROR - Could not power off node/s '{:?}'. Reason:\n{}",
                xname_vec,
                e.to_string()
            );

            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power off nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
}
