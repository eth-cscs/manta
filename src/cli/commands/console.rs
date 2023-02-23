use std::collections::HashSet;

use clap::ArgMatches;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_console: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    vault_base_url: String
) -> () {
    let included: HashSet<String>;
    let excluded: HashSet<String>;

    // User provided list of xnames to power reset
    let xnames: HashSet<String> = cli_console
        .get_one::<String>("XNAME")
        .unwrap()
        .replace(' ', "") // trim xnames by removing white spaces
        .split(',')
        .map(|xname| xname.to_string())
        .collect();

    let hsm_groups: Vec<crate::cluster_ops::ClusterDetails>;

    if hsm_group.is_some() {
        // hsm_group value provided
        hsm_groups =
            crate::cluster_ops::get_details(&shasta_token, shasta_base_url, hsm_group.unwrap()).await;

        // Take all nodes for all hsm_groups found and put them in a Set
        let hsm_groups_nodes = hsm_groups
            .iter()
            .flat_map(|hsm_group| {
                hsm_group
                    .members
                    .iter()
                    .map(|xname| xname.as_str().unwrap().to_string())
            })
            .collect();

        (included, excluded) =
            crate::node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

        if !excluded.is_empty() {
            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                    excluded,
                    hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                    hsm_groups_nodes);
            std::process::exit(-1);
        }
    } else {
        // no hsm_group value provided
        included = xnames.clone();
    }

    crate::node_console::connect_to_console(included.iter().next().unwrap(), vault_base_url).await.unwrap();
}
