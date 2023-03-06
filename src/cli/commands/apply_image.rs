use std::path::PathBuf;

use clap::ArgMatches;
use k8s_openapi::chrono;
use serde_yaml::Value;

use crate::shasta::cfs::configuration;

pub async fn exec(
    gitea_token: &str,
    gitea_base_url: &str,
    vault_base_url: String,
    hsm_group: Option<&String>,
    cli_apply_session: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) {
    println!("Now I am supposed to create an image ... ><!!!");

    if cli_apply_session.get_many::<PathBuf>("repo-path").is_some() {
        println!(
            "Create configuration from {} local repo",
            cli_apply_session
                .get_many::<PathBuf>("repo-path")
                .unwrap()
                .len()
        );
    } else if cli_apply_session.get_one::<PathBuf>("file").is_some() {
        println!("Create configuration from SAT file");

        let path_buf: &PathBuf = cli_apply_session.get_one("file").unwrap();
        let file_content = std::fs::read_to_string(path_buf.file_name().unwrap()).unwrap();
        let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

        // println!("\n### sat_input_file_yaml:\n{:#?}", sat_input_file_yaml);

        let configurations_yaml = sat_file_yaml["configurations"].as_sequence().unwrap();
        // println!("\n### configurations:\n{:#?}", configurations);

        if configurations_yaml.is_empty() {
            eprintln!("The input file has no configurations!");
            std::process::exit(-1);
        }

        if configurations_yaml.len() > 1 {
            eprintln!("Multiple CFS configurations found in input file, please clean the file so it only contains one.");
            std::process::exit(-1);
        }

        let configuration_yaml = &configurations_yaml[0];

        let configuration_name = configuration_yaml["name"]
            .as_str()
            .unwrap()
            .to_string()
            .replace(
                "__DATE__",
                &chrono::Utc::now().format("%Y%m%d%H%M%S").to_string(),
            );

        let cfs_configuration =
            configuration::CfsConfiguration::from_serde_yaml(&configuration_yaml);

        crate::shasta::cfs::configuration::http_client::put(
            shasta_token,
            shasta_base_url,
            &cfs_configuration,
            &configuration_name,
        )
        .await;

        log::info!("CFS configuration name: {}", configuration_name);

        let cfs_session = crate::shasta::cfs::session::CfsSession::new(
            configuration_name.clone(),
            configuration_name.clone(),
            None,
            *cli_apply_session
                .get_one::<u8>("ansible-verbosity")
                .unwrap(),
            true,
        );

        crate::shasta::cfs::session::http_client::post(shasta_token, shasta_base_url, &cfs_session)
            .await;

        log::info!("CFS session name: {}", cfs_session.name);
    }
}
