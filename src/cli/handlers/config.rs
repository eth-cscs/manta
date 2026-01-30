use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;
use crate::cli::commands;

pub async fn handle_config(
    cli_config: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    settings: &Config,
    cli: clap::Command,
) -> Result<(), Error> {
    if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
        commands::config_show::exec(backend, site_name, settings).await?
    } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
        if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
            commands::config_set_hsm::exec(cli_config_set_hsm, backend, site_name)
                .await?
        }
        if let Some(cli_config_set_parent_hsm) = cli_config_set.subcommand_matches("parent-hsm") {
            commands::config_set_parent_hsm::exec(
                cli_config_set_parent_hsm,
                backend,
                site_name,
            )
            .await?;
        }
        if let Some(cli_config_set_site) = cli_config_set.subcommand_matches("site") {
            commands::config_set_site::exec(cli_config_set_site).await?;
        }
        if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log") {
            commands::config_set_log::exec(cli_config_set_log).await?;
        }
    } else if let Some(cli_config_unset) = cli_config.subcommand_matches("unset") {
        if let Some(_cli_config_unset_hsm) = cli_config_unset.subcommand_matches("hsm") {
            commands::config_unset_hsm::exec().await?;
        }
        if let Some(_cli_config_unset_parent_hsm) = cli_config_unset.subcommand_matches("parent-hsm")
        {
            commands::config_unset_parent_hsm::exec(backend, site_name).await?;
        }
        if let Some(_cli_config_unset_auth) = cli_config_unset.subcommand_matches("auth") {
            commands::config_unset_auth::exec().await?;
        }
    } else if let Some(cli_config_generate_autocomplete) =
        cli_config.subcommand_matches("gen-autocomplete")
    {
        commands::config_gen_autocomplete::exec(
            cli,
            cli_config_generate_autocomplete,
        )
        .await?;
    }
    Ok(())
}
