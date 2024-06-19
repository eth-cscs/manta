use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    bss::{self, bootparameters::BootParameters},
    error::Error,
};

use crate::cli::commands::power_reset_nodes;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    kernel_params: &str,
    hsm_group_name_opt: Option<&Vec<String>>,
    xname_vec_opt: Option<&Vec<String>>,
) -> Result<(), Error> {
    println!("Set kernel parameters");

    let mut need_restart = false;

    let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name_vec,
        )
        .await
    } else if let Some(xname_vec) = xname_vec_opt {
        xname_vec.clone()
    } else {
        return Err(Error::Message(
            "Setting runtime configuration without a list of nodes".to_string(),
        ));
    };

    // Get current node boot params
    let current_node_boot_params: Vec<BootParameters> = bss::bootparameters::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xnames
            .iter()
            .map(|xname| xname.to_string())
            .collect::<Vec<String>>(),
    )
    .await
    .unwrap();

    // Update kernel parameters
    for mut boot_parameter in current_node_boot_params {
        boot_parameter.kernel = kernel_params.to_string();

        println!(
            "Updating '{:?}' kernel parameters to '{}'",
            boot_parameter.hosts, kernel_params
        );

        let component_patch_rep = mesa::bss::bootparameters::http_client::patch(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            &boot_parameter,
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );
    }

    Ok(())
}
