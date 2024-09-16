use std::collections::HashMap;

use comfy_table::{Cell, Table};
use mesa::bss::bootparameters::BootParameters;

pub fn print_table(
    boot_parameters_vec: Vec<BootParameters>,
    kernel_params_key_to_filter_opt: Option<String>,
) {
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

    let mut kernel_param_map: HashMap<String, Vec<String>> = HashMap::new();

    for boot_parameters in &boot_parameters_vec {
        let xname = boot_parameters.hosts.first().unwrap();
        let kernel_params = boot_parameters.params.clone();
        kernel_param_map
            .entry(kernel_params)
            .and_modify(|xname_vec| xname_vec.push(xname.clone()))
            .or_insert(vec![xname.clone()]);
    }

    let mut table = Table::new();

    table.set_header(vec!["XNAME", "Kernel Params"]);

    for (kernel_params, xname_vec) in kernel_param_map {
        let kernel_params_vec: Vec<String> = kernel_params
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

        let xnames = xname_vec.join("\n");

        table.add_row(vec![Cell::new(xnames), Cell::new(kernel_params_string)]);
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
