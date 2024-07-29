use mesa::{bss, error::Error, hsm};

use crate::common;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
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

    // Print table
    common::kernel_parameters_ops::print_table(boot_parameter_vec);

    Ok(())
}
