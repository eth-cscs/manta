use comfy_table::{Cell, Table};
use mesa::bss::bootparameters::BootParameters;

pub fn print_table(
    boot_parameters_vec: Vec<BootParameters>,
    kernel_params_key_to_filter_opt: Option<String>,
) {
    let kernel_params_key_vec: Vec<String> =
        if let Some(highlight) = kernel_params_key_to_filter_opt {
            highlight
                .split_whitespace()
                .map(|value| value.to_string())
                .collect()
        } else {
            vec![]
        };

    let mut table = Table::new();

    table.set_header(vec!["XNAME", "Kernel Params"]);

    for boot_parameters in boot_parameters_vec {
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
    }

    println!("{table}");
}
