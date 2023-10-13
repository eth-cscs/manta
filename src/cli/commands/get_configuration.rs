use mesa::{shasta::cfs, manta};

use crate::common::gitea;

pub async fn exec(
    gitea_base_url: &str,
    gitea_token: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&String>,
    // contains: Option<&String>,
    most_recent: Option<bool>,
    limit: Option<&u8>,
) {
    let cfs_configurations = manta::cfs::configuration::get_configuration(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        configuration_name,
        // contains,
        most_recent,
        limit,
    )
    .await;

    if cfs_configurations.is_empty() {
        println!("No CFS configuration found!");
        std::process::exit(0);
    } else if cfs_configurations.len() == 1 {
        let most_recent_cfs_configuration = &cfs_configurations[0];

        let mut layers: Vec<manta::cfs::configuration::Layer> = vec![];

        for layer in most_recent_cfs_configuration["layers"].as_array().unwrap() {
            let gitea_commit_details = gitea::http_client::get_commit_details(
                layer["cloneUrl"].as_str().unwrap(),
                layer["commit"].as_str().unwrap(),
                gitea_base_url,
                gitea_token,
            )
            .await
            .unwrap();

            layers.push(manta::cfs::configuration::Layer::new(
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

        manta::cfs::configuration::print_table(manta::cfs::configuration::Configuration::new(
            most_recent_cfs_configuration["name"].as_str().unwrap(),
            most_recent_cfs_configuration["lastUpdated"]
                .as_str()
                .unwrap(),
            layers,
        ));
    } else {
        cfs::configuration::utils::print_table(cfs_configurations);
    }
}
