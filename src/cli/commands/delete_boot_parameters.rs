use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::types::bss::BootParameters;

use crate::{
    common::authentication::get_api_token,
    manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    site_name: &str,
    hosts: Vec<String>,
) -> Result<(), anyhow::Error> {
    let shasta_token = get_api_token(backend, site_name).await?;

    let boot_parameters = BootParameters {
        hosts,
        macs: None,
        nids: None,
        params: "".to_string(),
        kernel: "".to_string(),
        initrd: "".to_string(),
        cloud_init: None,
    };

    let result = backend
        .delete_bootparameters(&shasta_token, &boot_parameters)
        .await;

    match result {
        Ok(_) => println!("Boot parameters deleted successfully"),
        Err(error) => eprintln!("{}", error),
    }

    Ok(())
}
