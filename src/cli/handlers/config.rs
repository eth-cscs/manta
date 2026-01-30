use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;
use crate::cli::parsers;

pub async fn handle_config(
    cli_config: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    settings: &Config,
    cli: clap::Command,
) -> Result<(), Error> {
    if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
        parsers::config::show::process_subcommand(backend, site_name, settings).await?
    } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
        if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
            parsers::config::set_hsm::process_subcommand(cli_config_set_hsm, backend, site_name)
                .await?
        }
        if let Some(cli_config_set_parent_hsm) = cli_config_set.subcommand_matches("parent-hsm") {
            parsers::config::set_parent_hsm::process_subcommand(
                cli_config_set_parent_hsm,
                backend,
                site_name,
            )
            .await?;
        }
        if let Some(cli_config_set_site) = cli_config_set.subcommand_matches("site") {
            parsers::config::set_site::process_subcommand(cli_config_set_site).await?;
        }
        if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log") {
            parsers::config::set_log::process_subcommand(cli_config_set_log).await?;
        }
    } else if let Some(cli_config_unset) = cli_config.subcommand_matches("unset") {
        if let Some(_cli_config_unset_hsm) = cli_config_unset.subcommand_matches("hsm") {
            parsers::config::unset_hsm::process_subcommand().await?;
        }
        if let Some(_cli_config_unset_parent_hsm) = cli_config_unset.subcommand_matches("parent-hsm")
        {
            parsers::config::unset_parent_hsm::process_subcommand(backend, site_name).await?;
        }
        if let Some(_cli_config_unset_auth) = cli_config_unset.subcommand_matches("auth") {
            parsers::config::unset_auth::process_subcommand().await?;
        }
    } else if let Some(cli_config_generate_autocomplete) =
        cli_config.subcommand_matches("gen-autocomplete")
    {
        parsers::config::generate_shell_autocompletion::process_subcommand(
            cli,
            cli_config_generate_autocomplete,
        )
        .await?;
    }
    Ok(())
}
