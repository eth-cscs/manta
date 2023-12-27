use mesa::cfs::{
    self,
    configuration::{
        mesa::r#struct::{Configuration, Layer},
        shasta::r#struct::cfs_configuration_response::CfsConfigurationResponse,
    },
};

use crate::common::{cfs_configuration_utils::print_table_struct, gitea};

pub async fn exec(
    gitea_base_url: &str,
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
            configuration_name,
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
                let gitea_commit_details = gitea::http_client::get_commit_details(
                    &layer.clone_url,
                    layer.commit.as_ref().unwrap(),
                    gitea_base_url,
                    gitea_token,
                )
                .await
                .unwrap();

                layers.push(Layer::new(
                    &layer.name,
                    layer
                        .clone_url
                        .trim_start_matches("https://api.cmn.alps.cscs.ch")
                        .trim_end_matches(".git"),
                    layer.commit.as_ref().unwrap(),
                    gitea_commit_details["commit"]["committer"]["name"]
                        .as_str()
                        .unwrap(),
                    gitea_commit_details["commit"]["committer"]["date"]
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
            print_table_struct(cfs_configuration_vec);
        }
    }
}
