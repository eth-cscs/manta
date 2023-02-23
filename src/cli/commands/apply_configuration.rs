use clap::ArgMatches;

use std::path::PathBuf;

use std::collections::HashSet;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_apply_configuration: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) {
    // * Parse input params
    let files: Vec<PathBuf> = cli_apply_configuration
            .get_many("repo-path")
            .unwrap()
            .cloned()
            .collect();

    // Parse hsm group
    let mut hsm_group_value = None;

    // Get hsm_group from cli arg
    if cli_apply_configuration.get_one::<String>("hsm-group").is_some() {
        hsm_group_value = cli_apply_configuration.get_one::<String>("hsm-group");
    }

    // Get hsm group from config file
    if hsm_group.is_some() {
        hsm_group_value = hsm_group;
    }
}
