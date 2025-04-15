use backend_dispatcher::{
    error::Error,
    interfaces::{
        bss::BootParametersTrait,
        hsm::{component::ComponentTrait, group::GroupTrait},
    },
    types::{BootParameters, Component},
};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, node_ops::resolve_node_list_user_input_to_xname_2},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    node_expression: &str,
    filter: Option<&String>,
    output: &str,
) -> Result<(), Error> {
    // Get BSS boot parameters

    // Convert user input to xname
    let xname_available_vec: Vec<String> = backend
        .get_group_available(shasta_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!(
                "ERROR - Could not get group list. Reason:\n{}",
                e.to_string()
            );
            std::process::exit(1);
        })
        .iter()
        .flat_map(|group| group.get_members())
        .collect();

    let node_metadata_vec: Vec<Component> = backend
        .get_all_nodes(shasta_token, Some("true"))
        .await
        .unwrap()
        .components
        .unwrap_or_default()
        .iter()
        .filter(|&node_metadata| xname_available_vec.contains(&node_metadata.id.as_ref().unwrap()))
        .cloned()
        .collect();

    let xname_vec =
        resolve_node_list_user_input_to_xname_2(node_expression, false, node_metadata_vec)
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
                    e
                );
                std::process::exit(1);
            });

    let boot_parameter_vec: Vec<BootParameters> = backend
        .get_bootparameters(shasta_token, &xname_vec)
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
