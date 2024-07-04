use mesa::{
    cfs::{
        self,
        configuration::mesa::r#struct::{
            cfs_configuration::{ConfigurationDetails, LayerDetails},
            cfs_configuration_response::v2::{CfsConfigurationResponse, Layer},
        },
    },
    common::gitea,
};
use serde_json::Value;

use crate::common::cfs_configuration_utils::print_table_struct;

pub async fn exec(
    gitea_base_url: &str,
    gitea_token: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&String>,
    configuration_name_pattern: Option<&String>,
    hsm_group_name_vec: &Vec<String>,
    limit: Option<&u8>,
    output_opt: Option<&String>,
) {
    let cfs_configuration_vec: Vec<CfsConfigurationResponse> =
        cfs::configuration::mesa::utils::get_and_filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            configuration_name.map(|elem| elem.as_str()),
            configuration_name_pattern.map(|elem| elem.as_str()),
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
            // Get CFS configuration details with data from VCS/Gitea
            let most_recent_cfs_configuration: &CfsConfigurationResponse =
                &cfs_configuration_vec[0];

            let mut layer_details_vec: Vec<LayerDetails> = vec![];

            for layer in &most_recent_cfs_configuration.layers {
                let layer_details: LayerDetails = get_configuration_layer_details(
                    shasta_root_cert,
                    gitea_base_url,
                    gitea_token,
                    layer.clone(),
                )
                .await;

                layer_details_vec.push(layer_details);
            }

            let (cfs_session_vec_opt, bos_sessiontemplate_vec_opt, image_vec_opt) =
                mesa::cfs::configuration::mesa::utils::get_derivatives(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &most_recent_cfs_configuration.name,
                )
                .await;

            crate::common::cfs_configuration_utils::print_table_details_struct(
                ConfigurationDetails::new(
                    &most_recent_cfs_configuration.name,
                    &most_recent_cfs_configuration.last_updated,
                    layer_details_vec,
                ),
                cfs_session_vec_opt,
                bos_sessiontemplate_vec_opt,
                image_vec_opt,
            );
        } else {
            print_table_struct(&cfs_configuration_vec);
        }
    }
}

pub async fn get_configuration_layer_details(
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
) -> LayerDetails {
    let commit_id: String = layer.commit.clone().unwrap_or("Not defined".to_string());
    // let branch_name_opt: Option<&str> = layer.branch.as_deref();
    // let mut most_recent_commit: bool = false;
    let mut branch_name: String = "".to_string();
    let mut tag_name: String = "".to_string();
    let commit_sha;

    let repo_ref_vec: Vec<Value> = gitea::http_client::get_all_refs_from_repo_url(
        gitea_base_url,
        gitea_token,
        &layer.clone_url,
        shasta_root_cert,
    )
    .await
    .unwrap();

    let mut ref_value_vec: Vec<&Value> = repo_ref_vec
        .iter()
        .filter(|repo_ref| {
            repo_ref
                .pointer("/object/sha")
                .unwrap()
                .as_str()
                .unwrap()
                .eq(&commit_id)
        })
        .collect();

    // Check if ref filtering returns an annotated tag, if so, then get the SHA of its
    // commit because it will be needed in case there are branches related to the
    // annotated tag
    if ref_value_vec.len() == 1 {
        // Potentially an annotated tag
        let ref_value = ref_value_vec.first().unwrap();
        log::debug!("Found ref in remote git repo:\n{:#?}", ref_value);

        let ref_type: &str = ref_value.pointer("/object/type").unwrap().as_str().unwrap();

        let mut r#ref = ref_value["ref"].as_str().unwrap().split("/").skip(1);

        let _ref_1 = r#ref.next();
        let ref_2 = r#ref.next();

        if ref_type == "tag" {
            // Yes, we are processing an annotated tag
            let tag_name = ref_2.unwrap();

            let commit_sha_value = gitea::http_client::get_commit_from_tag(
                ref_value["url"].as_str().unwrap(),
                &tag_name,
                gitea_token,
                shasta_root_cert,
            )
            .await
            .unwrap();

            commit_sha = commit_sha_value
                .pointer("/commit/sha")
                .unwrap()
                .as_str()
                .unwrap();

            let annotated_tag_commit_sha = [commit_id.clone(), commit_sha.to_string()];

            ref_value_vec = repo_ref_vec
                .iter()
                .filter(|repo_ref| {
                    let ref_sha: String = repo_ref
                        .pointer("/object/sha")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();

                    annotated_tag_commit_sha.contains(&ref_sha)
                })
                .collect();
        }
    }

    for ref_value in ref_value_vec {
        log::debug!("Found ref in remote git repo:\n{:#?}", ref_value);
        let ref_type: &str = ref_value.pointer("/object/type").unwrap().as_str().unwrap();
        let mut r#ref = ref_value["ref"].as_str().unwrap().split("/").skip(1);

        // let commit_sha_value: Value;

        let ref_1 = r#ref.next();
        // let ref_2 = r#ref.next();
        let ref_2 = r#ref.collect::<Vec<_>>().join("/");

        if ref_type == "commit" {
            // either branch or lightweight tag
            if let (Some("heads"), branch_name_aux) = (ref_1, ref_2.clone()) {
                // branch
                branch_name += &branch_name_aux;
            } else if let (Some("tags"), tag_name_aux) = (ref_1, ref_2) {
                // lightweight tag
                tag_name += &tag_name_aux;
            }

            // commit_sha = ref_value["url"].as_str().unwrap();
        } else {
            // annotated tag
            tag_name += &ref_2;

            /* commit_sha_value = gitea::http_client::get_commit_from_tag(
                ref_value["url"].as_str().unwrap(),
                &tag_name,
                gitea_token,
                shasta_root_cert,
            )
            .await
            .unwrap(); */

            /* commit_sha = commit_sha_value
            .pointer("/commit/sha")
            .unwrap()
            .as_str()
            .unwrap(); */
        }

        /* // check if layer commit is the most recent
        if commit_sha.eq(&layer.commit.clone().unwrap()) {
            // CFS layer commit is the same as the HEAD of the branch
            most_recent_commit = true;
        } */
    }

    if let Some(cfs_config_layer_branch) = &layer.branch {
        branch_name = cfs_config_layer_branch.to_string();
    }

    let commit_id_opt = layer.commit.as_ref();

    let gitea_commit_details: serde_json::Value = if let Some(commit_id) = commit_id_opt {
        gitea::http_client::get_commit_details_from_internal_url(
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

    LayerDetails::new(
        &layer.name,
        layer
            .clone_url
            .trim_start_matches("https://api.cmn.alps.cscs.ch")
            .trim_end_matches(".git"),
        &commit_id,
        gitea_commit_details
            .pointer("/commit/committer/name")
            .unwrap_or(&serde_json::json!("Not defined"))
            .as_str()
            .unwrap(),
        gitea_commit_details
            .pointer("/commit/committer/date")
            .unwrap_or(&serde_json::json!("Not defined"))
            .as_str()
            .unwrap(),
        &branch_name,
        &tag_name,
        &layer.playbook,
        // most_recent_commit,
    )
}
