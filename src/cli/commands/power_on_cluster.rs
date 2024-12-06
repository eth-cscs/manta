use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    common::jwt_ops::{self},
    error::Error,
    hsm,
};

use crate::common;

use infra_io::contracts::Power;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_arg_opt: &str,
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

    let backup_config_rslt = mesa::config::Config::new(
        &shasta_base_url.to_string(),
        Some(&shasta_token.to_string()),
        &shasta_root_cert.to_vec(),
        "csm", // FIXME: do not hardcode this value and move it to config file
    )
    .await;

    let backup_config = match backup_config_rslt {
        Ok(backup_config) => backup_config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let power_mgmt_summary_rslt = backup_config.power_on_sync(&xname_vec).await;

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
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on cluster {}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), hsm_group_name_arg_opt);
}
