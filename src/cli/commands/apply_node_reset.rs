use crate::common::node_ops;

use crate::shasta::capmc;

pub async fn exec(
    hsm_group: Option<&String>,
    // cli_apply_node_reset: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    xnames: Vec<&str>,
    reason: Option<&String>,
    force: bool,
) {
    // Get xnames from input param
    /* let xnames: Vec<&str> = cli_apply_node_reset
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

    let xname_list: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

    log::info!("Resetting servers: {:?}", xname_list);

    capmc::http_client::node_power_off::post_sync(
        shasta_token,
        shasta_base_url,
        // cli_apply_node_reset.get_one::<String>("reason"),
        // &xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        // *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
        xname_list.clone(),
        reason.cloned(),
        force,
    )
    .await
    .unwrap(); // TODO: idk why power on does not seems to work when forced

    let _node_reset_response = capmc::http_client::node_power_on::post(
        shasta_token,
        shasta_base_url,
        // cli_apply_node_reset.get_one::<String>("reason"),
        xname_list,
        reason.cloned(),
        false,
    )
    .await;
}
