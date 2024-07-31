use std::path::PathBuf;

use chrono::{DateTime, Local, Utc};
use comfy_table::Table;
use mesa::{
    bos::template::mesa::r#struct::v2::BosSessionTemplate,
    cfs::{
        configuration::mesa::r#struct::{
            cfs_configuration::ConfigurationDetails,
            cfs_configuration_request::v2::{CfsConfigurationRequest, Layer},
            cfs_configuration_response::v2::CfsConfigurationResponse,
        },
        session::mesa::r#struct::v2::CfsSessionGetResponse,
    },
    common::gitea,
    ims::image::r#struct::Image,
};
use substring::Substring;

use crate::common::local_git_repo;

pub fn print_table_struct(cfs_configurations: &Vec<CfsConfigurationResponse>) {
    let mut table = Table::new();

    table.set_header(vec!["Config Name", "Last updated", "Layers"]);

    for cfs_configuration in cfs_configurations {
        let mut layers: String = String::new();

        if !cfs_configuration.layers.is_empty() {
            let layers_json = &cfs_configuration.layers;

            layers = format!(
                "Name:     {}\nPlaybook: {}\nCommit:   {}",
                layers_json[0].name,
                layers_json[0].playbook,
                layers_json[0]
                    .commit
                    .as_ref()
                    .unwrap_or(&"Not defined".to_string()),
            );

            for layer in layers_json.iter().skip(1) {
                layers = format!(
                    "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
                    layers,
                    layer.name,
                    layer.playbook,
                    layer.commit.as_ref().unwrap_or(&"Not defined".to_string()),
                );
            }
        }

        table.add_row(vec![
            cfs_configuration.name.clone(),
            cfs_configuration
                .last_updated
                .clone()
                .parse::<DateTime<Local>>()
                .unwrap()
                .format("%d/%m/%Y %H:%M")
                .to_string(),
            layers,
        ]);
    }

    println!("{table}");
}

pub fn print_table_details_struct(
    cfs_configuration: ConfigurationDetails,
    cfs_session_vec_opt: Option<Vec<CfsSessionGetResponse>>,
    bos_sessiontemplate_vec_opt: Option<Vec<BosSessionTemplate>>,
    image_vec_opt: Option<Vec<Image>>,
) {
    let mut table = Table::new();

    table.set_header(vec![
        "Configuration Name",
        "Last updated",
        "Layers",
        "Derivatives",
    ]);

    let mut layers: String = String::new();

    for layer in cfs_configuration.config_layers {
        layers = format!(
            "{}\n\nName:     {}\nBranch:   {}\nTag:      {}\nDate:     {}\nAuthor:   {}\nCommit:   {}\nPlaybook: {}",
            layers,
            layer.name,
            layer.branch,
            /* if let true = layer.most_recent_commit {
                "(Up to date)"
            } else {
                "(Outdated)"
            }, */
            layer.tag,
            layer.commit_date,
            layer.author,
            layer.commit_id,
            layer.playbook
        );
    }

    let mut derivatives: String = String::new();

    if let Some(cfs_session_vec) = cfs_session_vec_opt {
        derivatives = derivatives + "CFS sessions:";
        for cfs_session in cfs_session_vec {
            derivatives = derivatives + "\n - " + &cfs_session.name.unwrap();
        }
    }

    if let Some(bos_sessiontemplate_vec) = bos_sessiontemplate_vec_opt {
        derivatives = derivatives + "\n\nBOS sessiontemplates:";
        for bos_sessiontemplate in bos_sessiontemplate_vec {
            derivatives = derivatives + "\n - " + &bos_sessiontemplate.name.unwrap();
        }
    }

    if let Some(image_vec) = image_vec_opt {
        derivatives = derivatives + "\n\nIMS images:";
        for image in image_vec {
            derivatives = derivatives + "\n - " + &image.name;
        }
    }

    layers = layers.trim_start_matches("\n\n").to_string();

    table.add_row(vec![
        cfs_configuration.name,
        cfs_configuration
            .last_updated
            .parse::<DateTime<Local>>()
            .unwrap()
            .format("%d/%m/%Y %H:%M")
            .to_string(),
        layers,
        derivatives,
    ]);

    println!("{table}");
}

pub async fn create_from_repos(
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_root_cert: &[u8],
    repos: Vec<PathBuf>,
    cfs_configuration_name: &String,
) -> CfsConfigurationRequest {
    // Create CFS configuration
    let mut cfs_configuration = CfsConfigurationRequest::new();
    cfs_configuration.name = cfs_configuration_name.to_string();

    for repo_path in &repos {
        // Get repo from path
        let repo = match local_git_repo::get_repo(&repo_path.to_string_lossy()) {
            Ok(repo) => repo,
            Err(_) => {
                eprintln!(
                    "Could not find a git repo in {}",
                    repo_path.to_string_lossy()
                );
                std::process::exit(1);
            }
        };

        // Get last (most recent) commit
        let local_last_commit = local_git_repo::get_last_commit(&repo).unwrap();

        // Get repo name
        let repo_ref_origin = repo.find_remote("origin").unwrap();

        log::info!("Repo ref origin URL: {}", repo_ref_origin.url().unwrap());

        let repo_ref_origin_url = repo_ref_origin.url().unwrap();

        let repo_name = repo_ref_origin_url.substring(
            repo_ref_origin_url.rfind(|c| c == '/').unwrap() + 1, // repo name should not include URI '/' separator
            repo_ref_origin_url.len(), // repo_ref_origin_url.rfind(|c| c == '.').unwrap(),
        );

        let api_url = "cray/".to_owned() + repo_name;

        // Check if repo and local commit id exists in Shasta cvs
        let shasta_commitid_details_resp =
            gitea::http_client::get_commit_details_from_internal_url(
                &api_url,
                // &format!("/cray/{}", repo_name),
                &local_last_commit.id().to_string(),
                gitea_token,
                shasta_root_cert,
            )
            .await;

        // Check sync status between user face and shasta VCS
        let shasta_commitid_details: serde_json::Value = match shasta_commitid_details_resp {
            Ok(_) => {
                log::debug!(
                    "Local latest commit id {} for repo {} exists in shasta",
                    local_last_commit.id(),
                    repo_name
                );
                shasta_commitid_details_resp.unwrap()
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

        let clone_url = gitea_base_url.to_owned() + "/cray/" + repo_name;

        // Create CFS layer
        let cfs_layer = Layer::new(
            clone_url,
            Some(shasta_commitid_details["sha"].as_str().unwrap().to_string()),
            format!(
                "{}-{}",
                repo_name.substring(0, repo_name.len()),
                chrono::offset::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ),
            String::from("site.yml"),
            None,
            None,
            None,
        );

        CfsConfigurationRequest::add_layer(&mut cfs_configuration, cfs_layer);
    }

    cfs_configuration
}
