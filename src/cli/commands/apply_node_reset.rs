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
    let included: HashSet<String>;
    let excluded: HashSet<String>;
    // Check andible limit matches the nodes in hsm_group
    let hsm_groups;

    let hsm_groups_nodes;

    // * Validate input params
    // Neither hsm_group (both config file or cli arg) nor xnames provided --> ERROR since we don't know the target nodes to apply the session to
    // NOTE: hsm group can be assigned either by config file or cli arg
    if cli_apply_node_reset.get_one::<String>("XNAMES").is_none()
        && hsm_group.is_none()
        && cli_apply_node_reset
            .get_one::<String>("hsm-group")
            .is_none()
    {
        // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
        eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
        std::process::exit(-1);
    }
    // * End validation input params

    // * Parse input params
    // Parse xnames
    // Get xnames nodes from cli arg
    // User provided list of xnames to power on
    let xnames: HashSet<String> = cli_apply_node_reset
        .get_one::<String>("XNAMES")
        .unwrap()
        .replace(' ', "") // trim xnames by removing white spaces
        .split(',')
        .map(|xname| xname.trim().to_string())
        .collect();

    // Check user has provided valid XNAMES
    let xname_re = Regex::new(r"^x\d{4}c\ds\db\dn\d$").unwrap();

    if xnames.iter().any(|xname| !xname_re.is_match(xname)) {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    // Parse hsm group
    let mut hsm_group_value = None;

    // Get hsm group from config file
    if hsm_group.is_some() {
        hsm_group_value = hsm_group;
    }
    // * End Parse input params

    // * Process/validate hsm group value (and ansible limit)
    if hsm_group_value.is_some() {
        // Get all hsm groups related to hsm_group input
        hsm_groups =
            cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group_value.unwrap())
                .await;

        // Take all nodes for all hsm_groups found and put them in a Vec
        hsm_groups_nodes = hsm_groups
            .iter()
            .flat_map(|hsm_group| {
                hsm_group
                    .members
                    .iter()
                    .map(|xname| xname.as_str().unwrap().to_string())
            })
            .collect();

        if !xnames.is_empty() {
            // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

            (included, excluded) =
                node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

            if !excluded.is_empty() {
                println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                                excluded,
                                hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                                hsm_groups_nodes);
                std::process::exit(-1);
            }
        } else {
            // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
            included = hsm_groups_nodes
        }
    } else {
        // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
        included = xnames
    }
    // * End Process/validate hsm group value (and ansible limit)

    let target_nodes: Vec<String> = included.into_iter().collect();

    log::info!("Servers to power reset: {:?}", target_nodes);

    capmc::http_client::node_power_off::post_sync(
        &shasta_token.to_string(),
        &shasta_base_url,
        cli_apply_node_reset.get_one::<String>("reason"),
        &target_nodes, // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
        *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
    )
    .await
    .unwrap(); // TODO: idk why power on does not seems to work when forced

    let _node_reset_response = capmc::http_client::node_power_on::post(
        shasta_token,
        shasta_base_url,
        cli_apply_node_reset.get_one::<String>("reason"),
        target_nodes,
        false,
    )
    .await;
}
