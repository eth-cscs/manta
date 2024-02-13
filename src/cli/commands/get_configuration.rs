use mesa::{
    cfs::{
        self,
        configuration::mesa::r#struct::{
            cfs_configuration::{Configuration, Layer},
            cfs_configuration_response::CfsConfigurationResponse,
        },
    },
    common::gitea,
};

use crate::common::cfs_configuration_utils::print_table_struct;

pub async fn exec(
    gitea_token: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&String>,
    hsm_group_name_vec: &Vec<String>,
    limit: Option<&u8>,
    output_opt: Option<&String>,
) {
    let cfs_configuration_vec: Vec<CfsConfigurationResponse> =
        cfs::configuration::mesa::http_client::get_and_filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            configuration_name.map(|elem| elem.as_str()),
            hsm_group_name_vec,
            limit,
        )
        .await;

    if cfs_configuration_vec.is_empty() {
        println!("No CFS configuration found!");
        std::process::exit(0);
    }

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&cfs_configuration_vec).unwrap()
        );
    } else {
        if cfs_configuration_vec.len() == 1 {
            let most_recent_cfs_configuration = &cfs_configuration_vec[0];

            let mut layers: Vec<Layer> = vec![];

            for layer in &most_recent_cfs_configuration.layers {
                let commit_id_opt = layer.commit.as_ref();

                let gitea_commit_details: serde_json::Value = if let Some(commit_id) = commit_id_opt
                {
                    gitea::http_client::get_commit_details(
                        &layer.clone_url,
                        commit_id,
                        gitea_token,
                        shasta_root_cert,
                    )
                    .await
                    .unwrap_or(serde_json::json!({}))
                } else {
                    serde_json::json!({})
                };

                layers.push(Layer::new(
                    &layer.name,
                    layer
                        .clone_url
                        .trim_start_matches("https://api.cmn.alps.cscs.ch")
                        .trim_end_matches(".git"),
                    layer.commit.as_ref().unwrap_or(&"Not defined".to_string()),
                    gitea_commit_details
                        .pointer("commit/committer/name")
                        .unwrap_or(&serde_json::json!("Not defined"))
                        .as_str()
                        .unwrap(),
                    gitea_commit_details
                        .pointer("commit/committer/date")
                        .unwrap_or(&serde_json::json!("Not defined"))
                        .as_str()
                        .unwrap(),
                ));
            }

            crate::common::cfs_configuration_utils::print_table_details_struct(Configuration::new(
                &most_recent_cfs_configuration.name,
                &most_recent_cfs_configuration.last_updated,
                layers,
            ));
        } else {
            print_table_struct(&cfs_configuration_vec);
        }
    }
}
