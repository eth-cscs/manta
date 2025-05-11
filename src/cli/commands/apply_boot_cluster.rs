use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  cli::commands::apply_boot_node, common::kafka::Kafka,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Updates boot params and desired configuration for all nodes that belongs to a HSM group
/// If boot params defined, then nodes in HSM group will be rebooted
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  new_boot_image_id_opt: Option<&String>,
  new_boot_image_configuration_opt: Option<&String>,
  new_runtime_configuration_opt: Option<&String>,
  new_kernel_parameters_opt: Option<&String>,
  hsm_group_name: &String,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
  kafka_audit_opt: Option<&Kafka>,
) {
  let xname_vec_rslt = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      vec![hsm_group_name.to_string()],
    )
    .await;

  let xname_vec = match xname_vec_rslt {
    Ok(xname_vec) => xname_vec,
    Err(e) => {
      eprintln!("Failed to get xnames from HSM group: {:?}", e);
      return;
    }
  };

  let result = apply_boot_node::exec(
    &backend,
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    new_boot_image_id_opt,
    new_boot_image_configuration_opt,
    new_runtime_configuration_opt,
    new_kernel_parameters_opt,
    &xname_vec.join(","),
    assume_yes,
    do_not_reboot,
    dry_run,
    kafka_audit_opt,
  )
  .await;

  if result.is_err() {
    eprintln!("Failed to apply boot node: {:?}", result);
    return;
  }
}
