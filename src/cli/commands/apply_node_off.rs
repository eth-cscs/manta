use mesa::shasta::capmc;

use crate::common::jwt_ops::get_claims_from_jwt_token;
use crate::common::node_ops;

pub async fn exec(
    hsm_group: Option<&String>,
    // cli_apply_node_off: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    xnames: Vec<&str>,
    reason: Option<String>,
    force: bool,
) {
    // Get xnames from input param
    /* let xnames: Vec<&str> = cli_apply_node_off
    .get_one::<String>("XNAMES")
    .unwrap()
    .split(',')
    .map(|xname| xname.trim())
    .collect(); */

    // Check user has provided valid XNAMES
    if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group).await {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    println!("Powering off servers: {:?}", xnames);

    capmc::http_client::node_power_off::post(
        shasta_token,
        shasta_base_url,
        // cli_apply_node_off.get_one::<String>("reason"),
        xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        reason,
        // *cli_apply_node_off.get_one::<bool>("force").unwrap(),
        force,
    )
    .await
    .unwrap();

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply nodes off {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xnames);
}
