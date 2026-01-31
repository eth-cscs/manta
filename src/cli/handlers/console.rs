use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use crate::common::{config::types::MantaConfiguration};
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
        let site = configuration
            .sites
            .get(&configuration.site.clone())
            .unwrap();
        console_node::exec(
            backend,
            site_name,
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
        let site = configuration
            .sites
            .get(&configuration.site.clone())
            .unwrap();
        console_cfs_session_image_target_ansible::exec(
            backend,
            site_name,
            settings_hsm_group_name_opt,
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
