use comfy_table::{Cell, ContentArrangement, Table};
use manta_backend_dispatcher::types::bss::BootParameters;

use crate::common::kernel_parameters_ops::group_boot_params_by_kernel_params;

/// Print kernel boot parameters grouped by common
/// parameter sets.
pub fn print_table(
  boot_parameters_vec: Vec<BootParameters>,
  kernel_params_key_to_filter_opt: Option<&str>,
) {
  let kernel_param_node_map = group_boot_params_by_kernel_params(
    &boot_parameters_vec,
    kernel_params_key_to_filter_opt,
  );

  let mut table = Table::new();

  table.set_header(vec!["XNAME", "Kernel Params"]);
  table.set_content_arrangement(ContentArrangement::Dynamic);

  for (kernel_params_vec, xname_vec) in kernel_param_node_map {
    let xnames = xname_vec.join("\n");

    table.add_row(vec![
      Cell::new(xnames),
      Cell::new(kernel_params_vec.join(" ")),
    ]);
  }

  println!("{table}");
}
