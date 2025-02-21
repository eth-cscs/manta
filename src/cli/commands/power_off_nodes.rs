use backend_dispatcher::interfaces::pcs::PCSTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, audit::Audit, jwt_ops, kafka::Kafka},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    /* shasta_base_url: &str,
    shasta_root_cert: &[u8], */
    hosts_string: &str,
    is_regex: bool,
    force: bool,
    assume_yes: bool,
    output: &str,
    kafka_audit_opt: Option<&Kafka>,
) {
    // Filter xnames to the ones members to HSM groups the user has access to
    //
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
        common::node_ops::nid_to_xname(backend, shasta_token, hosts_string, is_regex)
            .await
            .expect("Could not convert NID to XNAME")
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
    };

    if xname_vec.is_empty() {
        eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
        std::process::exit(0);
    } */

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

    /* let operation = if force { "force-off" } else { "soft-off" };

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
        .power_off_sync(shasta_token, &xname_vec, force)
        .await;

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
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "message": "power off"});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
        // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power off nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
    }
}
