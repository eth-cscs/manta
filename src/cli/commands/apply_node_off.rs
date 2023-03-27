use clap::ArgMatches;

use crate::common::node_ops;

use crate::shasta::capmc;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_apply_node_off: &ArgMatches,
    shasta_token: String,
    shasta_base_url: String,
) {
    // Get xnames from input param
    let xnames: Vec<&str> = cli_apply_node_off
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

    println!("Powering off servers: {:?}", xnames);

    capmc::http_client::node_power_off::post(
        shasta_token.to_string(),
        shasta_base_url,
        cli_apply_node_off.get_one::<String>("reason"),
        xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        *cli_apply_node_off.get_one::<bool>("force").unwrap(),
    )
    .await
    .unwrap();
}
