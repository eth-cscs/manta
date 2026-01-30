use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use crate::common::{config::types::MantaConfiguration, authentication::get_api_token};
use crate::cli::commands;

pub async fn handle_log(
    cli_log: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration: &MantaConfiguration,
) -> Result<(), Error> {
    let shasta_token = get_api_token(backend, site_name).await?;
    let user_input = cli_log
        .get_one::<String>("VALUE")
        .expect("ERROR - value is mandatory");
    let timestamps = cli_log.get_flag("timestamps");
    let group_available_vec = backend.get_group_name_available(&shasta_token).await?;
    let site = configuration
        .sites
        .get(&configuration.site.clone())
        .unwrap();
    let k8s_details = site
        .k8s
        .as_ref()
        .expect("ERROR - k8s section not found in configuration");
    match commands::log::exec(
        backend,
        site_name,
        &shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &group_available_vec,
        user_input,
        timestamps,
        k8s_details,
    )
    .await
    {
        Ok(_) => {
            println!("Log streaming ended");
            Ok(())
        }
        Err(e) => Err(Error::msg(e)),
    }
}
