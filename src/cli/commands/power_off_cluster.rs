use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::hsm;

use crate::common;

use infra_io::contracts::Power;

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

    let backend_config_rslt = mesa::config::Config::new(
        &shasta_base_url.to_string(),
        Some(&shasta_token.to_string()),
        &shasta_root_cert.to_vec(),
        "csm", // FIXME: do not hardcode this value and move it to config file
    )
    .await;

    let backend_config = match backend_config_rslt {
        Ok(iaas_ops) => iaas_ops,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let power_mgmt_summary_rslt = backend_config.power_off_sync(&xname_vec, force).await;

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
    let user = mesa::common::jwt_ops::get_name(shasta_token)
        .expect("ERROR - claim 'user' not found in JWT token");
    let username = mesa::common::jwt_ops::get_preferred_username(shasta_token)
        .expect("ERROR - claim 'preferred_uername' not found in JWT token");

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power off cluster {}", user, username, hsm_group_name_arg_opt);
}
