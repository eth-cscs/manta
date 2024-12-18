use mesa::hsm;

use crate::{backend_dispatcher::StaticBackendDispatcher, cli::commands::apply_boot_node};

/// Updates boot params and desired configuration for all nodes that belongs to a HSM group
/// If boot params defined, then nodes in HSM group will be rebooted
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    new_boot_image_id_opt: Option<&String>,
    new_boot_image_configuration_opt: Option<&String>,
    new_runtime_configuration_opt: Option<&String>,
    new_kernel_parameters_opt: Option<&String>,
    hsm_group_name: &String,
    assume_yes: bool,
    dry_run: bool,
    site_name: &str,
) {
    let xname_vec = hsm::group::utils::get_member_vec_from_hsm_group_name(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name,
    )
    .await;

    apply_boot_node::exec(
        backend,
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_boot_image_id_opt,
        new_boot_image_configuration_opt,
        new_runtime_configuration_opt,
        new_kernel_parameters_opt,
        xname_vec.iter().map(|xname| xname.as_str()).collect(),
        assume_yes,
        dry_run,
        site_name,
    )
    .await;
}