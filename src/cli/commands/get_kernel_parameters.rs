use mesa::{bss, error::Error};

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
    filter: Option<&String>,
    output: &str,
) -> Result<(), Error> {
    // Get BSS boot parameters
    let boot_parameter_vec = bss::bootparameters::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec,
    )
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
