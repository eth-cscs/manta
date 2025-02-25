use backend_dispatcher::{
    error::Error, interfaces::bss::BootParametersTrait, types::BootParameters,
};

use crate::backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    xnames: &String,
    _nids: Option<&String>,
    _macs: Option<&String>,
    _params: Option<&String>,
    _kernel: Option<&String>,
    _initrd: Option<&String>,
) -> Result<Vec<BootParameters>, Error> {
    println!("Get boot parameters");

    let hosts: Vec<String> = xnames.split(',').map(String::from).collect();
    /* let macs: Option<Vec<String>> = macs.map(|x| x.split(',').map(String::from).collect());
    let nids: Option<Vec<u32>> = nids.cloned().map(|x| {
        x.split(',')
            .map(|x| x.to_string().parse::<u32>().unwrap_or_default())
            .collect()
    });
    let params: String = params.cloned().unwrap_or_default().to_string();
    let kernel: String = kernel.cloned().unwrap_or_default().to_string();
    let initrd: String = initrd.cloned().unwrap_or_default().to_string();

    let boot_parameters = BootParameters {
        hosts: hosts.clone(),
        macs,
        nids,
        params,
        kernel,
        initrd,
        cloud_init: None,
    }; */

    backend.get_bootparameters(shasta_token, &hosts).await
}
