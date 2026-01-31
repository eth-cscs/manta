use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use crate::common::config::types::MantaConfiguration;
use crate::cli::commands;

pub async fn handle_log(
    cli_log: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration: &MantaConfiguration,
) -> Result<(), Error> {
    let user_input = cli_log
        .get_one::<String>("VALUE")
        .expect("ERROR - value is mandatory");
    let timestamps = cli_log.get_flag("timestamps");
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
        shasta_base_url,
        shasta_root_cert,
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
