use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use crate::common::{config::types::MantaConfiguration, authentication::get_api_token, authorization::get_groups_names_available};
use std::io::IsTerminal;
use crate::cli::commands::{console_node, console_cfs_session_image_target_ansible};

pub async fn handle_console(
    cli_console: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration: &MantaConfiguration,
    settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
    if let Some(cli_console_node) = cli_console.subcommand_matches("node") {
        if !std::io::stdout().is_terminal() {
            return Err(Error::msg(
                "This command needs to run in interactive mode. Exit",
            ));
        }
        let shasta_token = get_api_token(backend, site_name).await?;
        let site = configuration
            .sites
            .get(&configuration.site.clone())
            .unwrap();
        console_node::exec(
            backend,
            site_name,
            &shasta_token,
            cli_console_node.get_one::<String>("XNAME").unwrap(),
            site.k8s
                .as_ref()
                .expect("ERROR - k8s section not found in configuration"),
        )
        .await?;
    } else if let Some(cli_console_target_ansible) = cli_console.subcommand_matches("target-ansible") {
        if !std::io::stdout().is_terminal() {
            return Err(Error::msg(
                "This command needs to run in interactive mode. Exit",
            ));
        }
        let shasta_token = get_api_token(backend, site_name).await?;
        let target_hsm_group_vec = get_groups_names_available(
            backend,
            &shasta_token,
            None,
            settings_hsm_group_name_opt,
        )
        .await?;
        let site = configuration
            .sites
            .get(&configuration.site.clone())
            .unwrap();
        console_cfs_session_image_target_ansible::exec(
            backend,
            site_name,
            &target_hsm_group_vec,
            &shasta_token,
            shasta_base_url,
            shasta_root_cert,
            cli_console_target_ansible
                .get_one::<String>("SESSION_NAME")
                .unwrap(),
            site.k8s
                .as_ref()
                .expect("ERROR - k8s section not found in configuration"),
        )
        .await?;
    }
    Ok(())
}
