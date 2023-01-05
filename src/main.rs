mod gitea;

mod shasta;

use shasta::{
    authentication, capmc,
    cfs::{
        component as shasta_cfs_component, 
        session as shasta_cfs_session,
    },
};

mod shasta_cfs_session_logs;
mod create_cfs_session_from_repo;
mod local_git_repo;
mod manta;
mod node_console;
mod vault;
mod cli_programmatic;
mod cluster_ops;
mod node_ops;
mod config;

use termion::color;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {

    // Init logger
    env_logger::init();

    let settings = config::get("config");

    let shasta_base_url = settings.get::<String>("shasta_base_url").unwrap();
    let vault_base_url = settings.get::<String>("vault_base_url").unwrap();
    let gitea_base_url = settings.get::<String>("gitea_base_url").unwrap();
    let keycloak_base_url = settings.get::<String>("keycloak_base_url").unwrap();
    match settings.get::<String>("socks5_proxy") {
        Ok(socks_proxy) => std::env::set_var("SOCKS5", socks_proxy),
        Err(_) => log::info!("socks proxy not provided"),
    }

    let settings_hsm_group = settings.get::<String>("hsm_group");
    
    let hsm_group = match &settings_hsm_group {
        Ok(hsm_group_val) => {
            println!("\nWorking on nodes related to *{}{}{}* hsm groups\n", color::Fg(color::Green), hsm_group_val, color::Fg(color::Reset));
            Some(hsm_group_val)
        },
        Err(_) => None,
    };

    let shasta_token = authentication::get_api_token(&shasta_base_url, &keycloak_base_url).await?;

    let gitea_token = vault::http_client::fetch_shasta_vcs_token(&vault_base_url).await.unwrap();

    // Process input params
    let matches = cli_programmatic::get_matches(hsm_group);
    cli_programmatic::process_command(matches, shasta_token, shasta_base_url, vault_base_url, gitea_token, gitea_base_url, hsm_group).await;

    Ok(())
}
