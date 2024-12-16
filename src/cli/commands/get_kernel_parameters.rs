use backend_dispatcher::{contracts::BackendTrait, types::BootParameters};
use mesa::error::Error;

use crate::{backend_dispatcher::StaticBackendDispatcher, common};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
    filter: Option<&String>,
    output: &str,
) -> Result<(), Error> {
    // Get BSS boot parameters
    /* let boot_parameter_vec =
    bss::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, &xname_vec)
        .await
        .unwrap(); */

    let boot_parameter_vec: Vec<BootParameters> = backend
        .get_bootparameters(shasta_token, &xname_vec)
        .await
        .unwrap();

    match output {
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&boot_parameter_vec).unwrap()
        ),
        "table" => common::kernel_parameters_ops::print_table(boot_parameter_vec, filter),
        _ => panic!("ERROR - 'output' argument value missing or not supported"),
    }

    Ok(())
}
