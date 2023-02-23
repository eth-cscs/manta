use std::collections::HashSet;

use clap::ArgMatches;

pub async fn exec(
    gitea_token: &str,
    gitea_base_url: &str,
    vault_base_url: String,
    hsm_group: Option<&String>,
    cli_apply_session: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) -> () {
    let included: HashSet<String>;
    let excluded: HashSet<String>;
    // Check andible limit matches the nodes in hsm_group
    let hsm_groups;

    let cfs_configuration_name;

    let hsm_groups_nodes;

    // * Validate input params
    // Neither hsm_group (both config file or cli arg) nor ansible_limit provided --> ERROR since we don't know the target nodes to apply the session to
    // NOTE: hsm group can be assigned either by config file or cli arg
    if cli_apply_session
        .get_one::<String>("ansible-limit")
        .is_none()
        && hsm_group.is_none()
        && cli_apply_session.get_one::<String>("hsm-group").is_none()
    {
        // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
        eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
        std::process::exit(-1);
    }
    // * End validation input params

    // * Parse input params
    // Parse ansible limit
    // Get ansible limit nodes from cli arg
    let ansible_limit_nodes: HashSet<String> = if cli_apply_session
        .get_one::<String>("ansible-limit")
        .is_some()
    {
        // Get HashSet with all nodes from ansible-limit param
        cli_apply_session
            .get_one::<String>("ansible-limit")
            .unwrap()
            .replace(' ', "") // trim xnames by removing white spaces
            .split(',')
            .map(|xname| xname.to_string())
            .collect()
    } else {
        HashSet::new()
    };

    // Parse hsm group
    let mut hsm_group_value = None;

    // Get hsm_group from cli arg
    if cli_apply_session.get_one::<String>("hsm-group").is_some() {
        hsm_group_value = cli_apply_session.get_one::<String>("hsm-group");
    }

    // Get hsm group from config file
    if hsm_group.is_some() {
        hsm_group_value = hsm_group;
    }
    // * End Parse input params

    // * Process/validate hsm group value (and ansible limit)
    if hsm_group_value.is_some() {
        // Get all hsm groups related to hsm_group input
        hsm_groups =
            crate::cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group_value.unwrap())
                .await;

        //cfs_configuration_name = format!("{}-{}", hsm_group_value.unwrap(), cli_apply_session.get_one::<String>("name").unwrap());
        cfs_configuration_name = cli_apply_session
            .get_one::<String>("name")
            .unwrap()
            .to_string();

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

        if !ansible_limit_nodes.is_empty() {
            // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

            (included, excluded) =
                crate::node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, ansible_limit_nodes);

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
        cfs_configuration_name = cli_apply_session
            .get_one::<String>("name")
            .unwrap()
            .to_string();
        included = ansible_limit_nodes
    }
    // * End Process/validate hsm group value (and ansible limit)

    // * Create CFS session
    let cfs_session_name = crate::create_cfs_session_from_repo::run(
        &cfs_configuration_name,
        cli_apply_session
            .get_many("repo-path")
            .unwrap()
            .cloned()
            .collect(),
        // vec![cli_apply_session
        //     .get_one::<String>("repo-path")
        //     .unwrap()
        //     .to_string()],
        gitea_token,
        gitea_base_url,
        &shasta_token,
        &shasta_base_url,
        included.into_iter().collect::<Vec<String>>().join(","), // Convert Hashset to String with comma separator, need to convert to Vec first following https://stackoverflow.com/a/47582249/1918003
        cli_apply_session
            .get_one::<String>("ansible-verbosity")
            .unwrap()
            .parse()
            .unwrap(),
    )
    .await;

    let watch_logs = cli_apply_session.get_one::<bool>("watch-logs");

    if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");
        crate::shasta_cfs_session_logs::client::session_logs(
            vault_base_url,
            cfs_session_name.unwrap().as_str(),
            None,
        )
        .await.unwrap();
    }
    // * End Create CFS session
}
