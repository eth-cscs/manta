use anyhow::Error;
use clap::Command;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use ::config::Config;
use crate::MantaConfiguration;
use crate::Kafka;

use crate::cli::handlers::{config, power, add, update, get, apply, log, console, migrate, delete, misc};

pub async fn process_cli(
    cli: Command,
    backend: StaticBackendDispatcher,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: Option<&String>,
    gitea_base_url: &str,
    settings_hsm_group_name_opt: Option<&String>,
    k8s_api_url: Option<&String>,
    kafka_audit_opt: Option<&Kafka>,
    settings: &Config,
    configuration: &MantaConfiguration,
) -> Result<(), Error> {
    let site_name: String = settings
        .get("site")
        .map_err(|_| Error::msg("'site' value in configuration file is missing or does not have a value. Exit"))?;

    let cli_root = cli.clone().get_matches();

    if let Some(cli_config) = cli_root.subcommand_matches("config") {
        config::handle_config(cli_config, &backend, &site_name, settings, cli).await?;
    } else {
        if let Some(cli_power) = cli_root.subcommand_matches("power") {
            power::handle_power(
                cli_power,
                &backend,
                &site_name,
                settings_hsm_group_name_opt,
                kafka_audit_opt,
            )
            .await?;
        } else if let Some(cli_add) = cli_root.subcommand_matches("add") {
            add::handle_add(
                cli_add,
                &backend,
                &site_name,
                settings_hsm_group_name_opt,
                kafka_audit_opt,
            )
            .await?;
        } else if let Some(_cli_update) = cli_root.subcommand_matches("update") {
            update::handle_update(&cli_root, &backend, &site_name, kafka_audit_opt).await?; // Pass cli_root for now as we might need better matching
        } else if let Some(cli_get) = cli_root.subcommand_matches("get") {
            get::handle_get(
                cli_get,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url,
                gitea_base_url,
                settings_hsm_group_name_opt,
            )
            .await?;
        } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
            apply::handle_apply(
                cli_apply,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url.unwrap(),
                gitea_base_url,
                settings_hsm_group_name_opt,
                k8s_api_url.unwrap(),
                kafka_audit_opt,
                settings,
                configuration,
            )
            .await?;
        } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
            log::handle_log(
                cli_log,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                configuration,
            )
            .await?;
        } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
            console::handle_console(
                cli_console,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                configuration,
                settings_hsm_group_name_opt,
            )
            .await?;
        } else if let Some(cli_migrate) = cli_root.subcommand_matches("migrate") {
            migrate::handle_migrate(
                cli_migrate,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                settings_hsm_group_name_opt,
                kafka_audit_opt,
            )
            .await?;
        } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
            delete::handle_delete(
                cli_delete,
                &backend,
                &site_name,
                shasta_base_url,
                shasta_root_cert,
                settings_hsm_group_name_opt,
                kafka_audit_opt,
            )
            .await?;
        } else {
            misc::handle_misc(
                &cli_root,
                &backend,
                &site_name,
                shasta_root_cert,
                vault_base_url,
                gitea_base_url,
                settings_hsm_group_name_opt,
                kafka_audit_opt,
            )
            .await?;
        }
    }

    Ok(())
}
