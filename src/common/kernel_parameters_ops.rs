use comfy_table::{Cell, Table};
use mesa::bss::bootparameters::BootParameters;

pub fn print_table(boot_parameters_vec: Vec<BootParameters>) {
    let mut table = Table::new();

    table.set_header(vec!["XNAME", "Kernel Params"]);

    for boot_parameters in boot_parameters_vec {
        let kernel_params_vec: Vec<&str> = boot_parameters.params.split_whitespace().collect();
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

            kernel_params_string.push_str(kernel_param);
        }

        let xname = boot_parameters.hosts.first().unwrap().clone();

        table.add_row(vec![Cell::new(xname), Cell::new(kernel_params_string)]);
    }

    println!("{table}");
}
