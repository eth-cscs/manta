use mesa::{common::jwt_ops::get_claims_from_jwt_token, error::Error, pcs};

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: &Vec<String>,
    reason_opt: Option<String>,
) {
    let operation = "on";

    let power_mgmt_summary_rslt = pcs::transitions::http_client::post_block(
        shasta_base_url,
        shasta_token,
        shasta_root_cert,
        operation,
        xname_vec,
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

    common::pcs_utils::print_summary_table(power_mgmt_summary);

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power reset nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);

    /* // Check Nodes are shutdown
    let _ = capmc::http_client::node_power_status::post(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec,
    )
    .await
    .unwrap();

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);

    let _ = wait_nodes_to_power_on(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname_vec,
        reason_opt,
    )
    .await; */
}
