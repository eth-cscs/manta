use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;
use crate::common::{config::types::MantaConfiguration, kafka::Kafka};
use crate::cli::parsers;

pub async fn handle_apply(
    cli_apply: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    gitea_base_url: &str,
    settings_hsm_group_name_opt: Option<&String>,
    k8s_api_url: &str,
    kafka_audit_opt: Option<&Kafka>,
    settings: &Config,
    configuration: &MantaConfiguration,
) -> Result<(), Error> {
    parsers::apply::parse_subcommand(
        cli_apply,
        backend.clone(),
        site_name,
        shasta_base_url,
        shasta_root_cert,
        vault_base_url,
        gitea_base_url,
        settings_hsm_group_name_opt,
        k8s_api_url,
        kafka_audit_opt,
        settings,
        configuration,
    )
    .await?;
    Ok(())
}
