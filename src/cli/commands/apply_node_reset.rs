use std::collections::HashSet;

use clap::ArgMatches;
use regex::Regex;

use crate::common::{cluster_ops, node_ops};

use crate::shasta::capmc;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_apply_node_reset: &ArgMatches,
    shasta_token: String,
    shasta_base_url: String,
) {
    // Get xnames from input param
    let xnames: Vec<&str> = cli_apply_node_reset
        .get_one::<String>("XNAMES")
        .unwrap()
        .split(',')
        .map(|xname| xname.trim())
        .collect();

    // Check user has provided valid XNAMES
    if !node_ops::validate_xnames(&shasta_token, &shasta_base_url, &xnames, hsm_group).await {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    log::info!("Resetting servers: {:?}", xnames);

    capmc::http_client::node_power_off::post_sync(
        &shasta_token.to_string(),
        &shasta_base_url,
        cli_apply_node_reset.get_one::<String>("reason"),
        &xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
    )
    .await
    .unwrap(); // TODO: idk why power on does not seems to work when forced

    let _node_reset_response = capmc::http_client::node_power_on::post(
        shasta_token,
        shasta_base_url,
        cli_apply_node_reset.get_one::<String>("reason"),
        xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        false,
    )
    .await;
}
