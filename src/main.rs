mod cli;
mod common;
mod config;

use mesa::shasta;
use std::path::PathBuf;

use directories::ProjectDirs;

use shasta::authentication;

use crate::common::log_ops;

// DHAT (profiling)
// #[cfg(feature = "dhat-heap")]
// #[global_allocator]
// static ALOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
    // DHAT (profiling)
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path_to_manta_configuration_file = PathBuf::from(project_dirs.unwrap().config_dir());

    path_to_manta_configuration_file.push("config"); // ~/.config/manta/config is the file

    log::info!(
        "Reading manta configuration from {}",
        &path_to_manta_configuration_file.to_string_lossy()
    );

    let settings = config::get_configuration(&path_to_manta_configuration_file.to_string_lossy());

    let shasta_base_url = settings.get::<String>("shasta_base_url").unwrap();
    let vault_base_url = settings.get::<String>("vault_base_url").unwrap();
    let vault_role_id = settings.get::<String>("vault_role_id").unwrap();
    let vault_secret_path = settings.get::<String>("vault_secret_path").unwrap();
    let gitea_base_url = settings.get::<String>("gitea_base_url").unwrap();
    let keycloak_base_url = settings.get::<String>("keycloak_base_url").unwrap();
    let k8s_api_url = settings.get::<String>("k8s_api_url").unwrap();
    let log_level = settings.get::<String>("log").unwrap_or("error".to_string());

    // Init logger
    // env_logger::init();
    // log4rs::init_file("log4rs.yml", Default::default()).unwrap(); // log4rs file configuration
    log_ops::configure(log_level); // log4rs programatically configuration

    if let Ok(socks_proxy) = settings.get::<String>("socks5_proxy") {
        std::env::set_var("SOCKS5", socks_proxy);
    }

    let settings_hsm_group = settings.get::<String>("hsm_group");
    // let base_image_id = settings.get::<String>("base_image_id").unwrap();

    let hsm_group = match &settings_hsm_group {
        Ok(hsm_group_val) => {
            /* println!(
                "\nWorking on nodes related to *{}{}{}* hsm groups\n",
                color::Fg(color::Green),
                hsm_group_val,
                color::Fg(color::Reset)
            ); */
            Some(hsm_group_val)
        }
        Err(_) => None,
    };

    let shasta_token = authentication::get_api_token(&shasta_base_url, &keycloak_base_url).await?;

    let gitea_token =
        crate::common::vault::http_client::fetch_shasta_vcs_token(&vault_base_url, &vault_secret_path, &vault_role_id)
            .await
            .unwrap();

    // Process input params
    let matches = crate::cli::build::build_cli(hsm_group).get_matches();
    let cli_result = crate::cli::process::process_cli(
        matches,
        &shasta_token,
        &shasta_base_url,
        &vault_base_url,
        &vault_secret_path,
        &vault_role_id,
        &gitea_token,
        &gitea_base_url,
        hsm_group,
        // &base_image_id,
        &k8s_api_url,
    )
    .await;

    match cli_result {
        Ok(_) => Ok(()),
        Err(e) => panic!("{}", e),
    }
}
