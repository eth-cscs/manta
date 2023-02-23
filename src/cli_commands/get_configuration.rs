use clap::ArgMatches;

pub async fn exec(
    gitea_token: &str,
    hsm_group: Option<&String>,
    cli_get_configuration: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) -> () {
    let configuration_name = cli_get_configuration.get_one::<String>("name");

    let hsm_group_name = match hsm_group {
        // ref: https://stackoverflow.com/a/32487173/1918003
        None => cli_get_configuration.get_one::<String>("hsm-group"),
        Some(hsm_group_val) => Some(hsm_group_val),
    };

    let most_recent = cli_get_configuration.get_one::<bool>("most-recent");

    let limit_number;

    if let Some(true) = most_recent {
        limit_number = Some(&1);
    } else if let Some(false) = most_recent {
        limit_number = cli_get_configuration.get_one::<u8>("limit");
    } else {
        limit_number = None;
    }

    // Get CFS configurations
    let cfs_configurations = crate::shasta::cfs::configuration::http_client::get(
        &shasta_token,
        &shasta_base_url,
        hsm_group_name,
        configuration_name,
        limit_number,
    )
    .await
    .unwrap_or(Vec::new());

    if cfs_configurations.is_empty() {
        println!("No CFS configuration found!");
        std::process::exit(0);
    } else if cfs_configurations.len() == 1 {
        let most_recent_cfs_configuration = &cfs_configurations[0];

        let mut layers: Vec<crate::manta::cfs::configuration::Layer> = vec![];

        for layer in most_recent_cfs_configuration["layers"].as_array().unwrap() {
            let gitea_commit_details = crate::gitea::http_client::get_commit_details(
                layer["cloneUrl"].as_str().unwrap(),
                layer["commit"].as_str().unwrap(),
                gitea_token,
            )
            .await
            .unwrap();

            layers.push(crate::manta::cfs::configuration::Layer::new(
                layer["name"].as_str().unwrap(),
                layer["cloneUrl"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("https://api.cmn.alps.cscs.ch")
                    .trim_end_matches(".git"),
                layer["commit"].as_str().unwrap(),
                gitea_commit_details["commit"]["committer"]["name"]
                    .as_str()
                    .unwrap(),
                gitea_commit_details["commit"]["committer"]["date"]
                    .as_str()
                    .unwrap(),
            ));
        }

        crate::manta::cfs::configuration::print_table(
            crate::manta::cfs::configuration::Configuration::new(
                most_recent_cfs_configuration["name"].as_str().unwrap(),
                most_recent_cfs_configuration["lastUpdated"]
                    .as_str()
                    .unwrap(),
                layers,
            ),
        );
    } else {
        crate::shasta::cfs::configuration::utils::print_table(cfs_configurations);
    }
}
