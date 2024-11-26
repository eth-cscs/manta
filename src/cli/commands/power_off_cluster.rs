use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{error::Error, hsm, pcs};

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_arg_opt: &str,
    force: bool,
    assume_yes: bool,
    output: &str,
) {
    let xname_vec = hsm::group::utils::get_member_vec_from_hsm_group_name(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name_arg_opt,
    )
    .await;

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
    let user = mesa::common::jwt_ops::get_name(shasta_token)
        .expect("ERROR - claim 'user' not found in JWT token");
    let username = mesa::common::jwt_ops::get_preferred_username(shasta_token)
        .expect("ERROR - claim 'preferred_uername' not found in JWT token");

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power off cluster {}", user, username, hsm_group_name_arg_opt);
}
