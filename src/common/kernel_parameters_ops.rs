use std::collections::HashMap;

use comfy_table::{Cell, ContentArrangement, Table};
use manta_backend_dispatcher::types::bss::BootParameters;

pub fn print_table(
  boot_parameters_vec: Vec<BootParameters>,
  kernel_params_key_to_filter_opt: Option<&String>,
) {
  // Get list of key value pairs from kernel params to filter
  let mut kernel_params_key_vec: Vec<String> =
    if let Some(highlight) = kernel_params_key_to_filter_opt {
      highlight
        .split(",")
        .map(|value| value.trim().to_string())
        .collect()
    } else {
      vec![]
    };

  // Sort kernel params
  kernel_params_key_vec.sort();

  // Get list of key value pairs from kernel params per node
  let mut kernel_param_node_map: HashMap<Vec<String>, Vec<String>> =
    HashMap::new();

  for boot_parameters in &boot_parameters_vec {
    let mut host_vec = boot_parameters.hosts.clone();
    let kernel_params = boot_parameters.params.clone();

    // Get list of key value pairs from kernel params per node
    let kernel_params_vec: Vec<String> = kernel_params
      .split_whitespace()
      .map(|value| value.to_string())
      .collect();

    // Filter kernel params
    let mut kernel_params_vec: Vec<String> = if !kernel_params_key_vec
      .is_empty()
    {
      kernel_params_vec
        .into_iter()
        .filter(|kp| kernel_params_key_vec.iter().any(|kp_k| kp.contains(kp_k)))
        .collect()
    } else {
      kernel_params_vec.clone()
    };

    // Sort kernel params
    kernel_params_vec.sort();

    kernel_param_node_map
      .entry(kernel_params_vec)
      .and_modify(|xname_vec| xname_vec.append(&mut host_vec))
      .or_insert(host_vec);
  }

  // Create table and format data
  let mut table = Table::new();

  table.set_header(vec!["XNAME", "Kernel Params"]);
  table.set_content_arrangement(ContentArrangement::Dynamic);

  // Format kernel params in table cell
  for (kernel_params_vec, xname_vec) in kernel_param_node_map {
    /* let cell_max_width = kernel_params_vec
      .iter()
      .map(|value| value.len())
      .max()
      .unwrap_or(0);

    let mut kernel_params_string: String = if !kernel_params_vec.is_empty() {
      kernel_params_vec[0].to_string()
    } else {
      "".to_string()
    };

    let mut cell_width = kernel_params_string.len();

    for kernel_param in kernel_params_vec.iter().skip(1) {
      cell_width += kernel_param.len();

      if cell_width + kernel_param.len() >= cell_max_width {
        kernel_params_string.push_str("\n");
        cell_width = 0;
      } else {
        kernel_params_string.push_str(" ");
      }

      kernel_params_string.push_str(&kernel_param);
    } */

    let xnames = xname_vec.join("\n");

    // table.add_row(vec![Cell::new(xnames), Cell::new(kernel_params_string)]);
    table.add_row(vec![
      Cell::new(xnames),
      Cell::new(kernel_params_vec.join(" ")),
    ]);
  }

  /* for boot_parameters in boot_parameters_vec {
      let kernel_params_vec: Vec<String> = boot_parameters
          .params
          .split_whitespace()
          .map(|value| value.to_string())
          .collect();

      let kernel_params_vec: Vec<String> = if !kernel_params_key_vec.is_empty() {
          kernel_params_vec
              .into_iter()
              .filter(|kp| kernel_params_key_vec.iter().any(|kp_k| kp.contains(kp_k)))
              .collect()
      } else {
          kernel_params_vec.clone()
      };

      let cell_max_width = kernel_params_vec
          .iter()
          .map(|value| value.len())
          .max()
          .unwrap_or(0);

      let mut kernel_params_string: String = kernel_params_vec[0].to_string();
      let mut cell_width = kernel_params_string.len();

      for kernel_param in kernel_params_vec.iter().skip(1) {
          cell_width += kernel_param.len();

          if cell_width + kernel_param.len() >= cell_max_width {
              kernel_params_string.push_str("\n");
              cell_width = 0;
          } else {
              kernel_params_string.push_str(" ");
          }

          kernel_params_string.push_str(&kernel_param);
      }

      let xname = boot_parameters.hosts.first().unwrap().clone();

      table.add_row(vec![Cell::new(xname), Cell::new(kernel_params_string)]);
  } */

  println!("{table}");
}
