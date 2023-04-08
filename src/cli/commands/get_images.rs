use clap::ArgMatches;
use comfy_table::Table;

use crate::{common, shasta::bos};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_get_image: &ArgMatches,
    xnames: Option<Vec<&str>>,
    cfs_configuration_name: Option<&String>,
    hsm_group: Option<&String>,
) {
    // Get BOS sessiontemplates for the hsm group
    let bos_sessiontemplates_resp =
        bos::template::http_client::get(shasta_token, shasta_base_url, hsm_group, None, None)
            .await
            .unwrap();

    let mut images_details = Vec::new();

    for bos_sessiontemplate in bos_sessiontemplates_resp {
        let compute_type = if bos_sessiontemplate.pointer("/boot_sets/compute").is_some() {
            "compute"
        } else {
            "uan"
        };

        let image_id = bos_sessiontemplate
            .pointer(&("/boot_sets/".to_owned() + compute_type + "/path"))
            .and_then(|image_id_value| image_id_value.as_str())
            .unwrap_or_default()
            .trim_start_matches("s3://boot-images/")
            .trim_end_matches("/manifest.json");
        let cfs_configuration = bos_sessiontemplate
            .pointer("/cfs/configuration")
            .and_then(|cfs_configuration_value| cfs_configuration_value.as_str())
            .unwrap_or_default();
        let hsm_groups = bos_sessiontemplate
            .pointer("/boot_sets/compute/node_groups")
            .unwrap_or(&serde_json::Value::Array(Vec::new()))
            .as_array()
            .unwrap()
            .iter()
            .map(|hsm_group| hsm_group.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        let node_list = bos_sessiontemplate
            .pointer("/boot_sets/compute/node_list")
            .and_then(|node_list_value| node_list_value.as_array());

        images_details.push(vec![
            image_id.to_owned(),
            compute_type.to_string(),
            cfs_configuration.to_owned(),
            hsm_groups,
            common::node_ops::nodes_to_string_format_discrete_columns(node_list, 4),
        ]);
    }

    let mut table = Table::new();

    table.set_header(vec!["Image ID", "Type", "CFS configuration", "HSM groups", "Nodes"]);

    for image in images_details {
        table.add_row(image);
    }

    println!("{table}");
}
