use backend_dispatcher::interfaces::hsm::group::GroupTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    common::jwt_ops::{self},
    error::Error,
    pcs,
};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, audit::Audit, kafka::Kafka},
};

pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_arg: &str,
    assume_yes: bool,
    output: &str,
    kafka_audit: &Kafka,
) {
    let xname_vec = backend
        .get_member_vec_from_group_name_vec(
            shasta_token,
            /* shasta_base_url,
            shasta_root_cert, */
            vec![hsm_group_name_arg.to_string()],
        )
        .await
        .unwrap();
    /* let xname_vec = hsm::group::utils::get_member_vec_from_hsm_group_name(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name_arg_opt,
    )
    .await; */

    if !assume_yes {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{:?}\nThe nodes above will be powered on. Please confirm to proceed?",
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

    let operation = "on";

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
                "ERROR - Could not power on node/s '{:?}'. Reason:\n{}",
                xname_vec,
                e.to_string()
            );

            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "group": hsm_group_name_arg, "message": "power on"});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
    // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on cluster {}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), hsm_group_name_arg);
}
