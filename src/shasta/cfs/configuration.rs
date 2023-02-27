use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    #[serde(rename = "cloneUrl")]
    clone_url: String,
    commit: Option<String>,
    name: String,
    playbook: String,
    branch: Option<String>,
}

#[derive(Debug, Serialize)] // TODO: investigate why serde can Deserialize dynamically syzed structs `Vec<Layer>`
pub struct CfsConfiguration {
    layers: Vec<Layer>,
}

impl Layer {
    pub fn new(
        clone_url: String,
        commit: Option<String>,
        name: String,
        playbook: String,
        branch: Option<String>,
    ) -> Self {
        Self {
            clone_url,
            commit,
            name,
            playbook,
            branch,
        }
    }

    pub fn new_from_product(
        gitea_base_url: String,
        repo_name: String,
        name: String,
        playbook: String,
        branch: Option<String>,
    ) -> Self {
        let clone_url = format!("{}/cray/{}", gitea_base_url, repo_name);
        let commit = None;
        Self {
            clone_url,
            commit,
            name,
            playbook,
            branch,
        }
    }

    pub fn new_from_git(
        gitea_base_url: String,
        repo_name: String,
        name: String,
        playbook: String,
        branch: Option<String>,
    ) -> Self {
        let clone_url = format!("{}/cray/{}", gitea_base_url, repo_name);
        let commit = None;
        Self {
            clone_url,
            commit,
            name,
            playbook,
            branch,
        }
    }
}

impl CfsConfiguration {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }
}

pub fn add_layer(layer: Layer, mut configuration: CfsConfiguration) -> CfsConfiguration {
    configuration.layers.push(layer);
    configuration
}

pub mod http_client {

    use std::error::Error;

    use super::CfsConfiguration;
    use serde_json::Value;

    pub async fn put(
        shasta_token: &str,
        shasta_base_url: &str,
        configuration: &CfsConfiguration,
        configuration_name: &str,
    ) -> Result<Value, Box<dyn Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let mut api_url = shasta_base_url.clone().to_string();
        api_url.push_str("/cfs/v2/configurations/");
        api_url.push_str(configuration_name);

        let resp = client
            .put(api_url)
            // .put(format!("{}{}{}", shasta_base_url, "/cfs/v2/configurations/", configuration_name))
            .json(configuration)
            .bearer_auth(shasta_token)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(serde_json::from_str(&resp.text().await?)?)
        } else {
            Err(resp.json::<Value>().await?["detail"]
                .as_str()
                .unwrap()
                .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: Option<&String>,
        configuration_name: Option<&String>,
        limit_number: Option<&u8>,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let mut api_url = shasta_base_url.clone().to_string();
        api_url.push_str("/cfs/v2/configurations");

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/cfs/v2/configurations"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value = if resp.status().is_success() {
            serde_json::from_str(&resp.text().await?)?
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        };

        let mut cluster_cfs_configs = json_response.as_array().unwrap().clone();

        if hsm_group_name.is_some() {
            cluster_cfs_configs.retain(|cfs_configuration| {
                cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .contains(hsm_group_name.unwrap())
            });
        }

        if configuration_name.is_some() {
            cluster_cfs_configs.retain(|cfs_configuration| {
                cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .eq(configuration_name.unwrap())
            });
        }

        cluster_cfs_configs.sort_by(|a, b| {
            a["lastUpdated"]
                .as_str()
                .unwrap()
                .cmp(b["lastUpdated"].as_str().unwrap())
        });

        if limit_number.is_some() {
            // Limiting the number of results to return to client

            cluster_cfs_configs = cluster_cfs_configs[cluster_cfs_configs
                .len()
                .saturating_sub(*limit_number.unwrap() as usize)..]
                .to_vec();
        }

        Ok(cluster_cfs_configs)
    }
}

pub mod utils {

    use crate::common::gitea;
    use crate::common::local_git_repo;
    use crate::shasta::cfs::configuration;
    use comfy_table::Table;
    use k8s_openapi::chrono;
    use serde_json::Value;
    use substring::Substring;

    pub fn print_table(cfs_configurations: Vec<Value>) {
        let mut table = Table::new();

        table.set_header(vec!["Name", "Last updated", "Layers"]);

        for cfs_configuration in cfs_configurations {
            let mut layers: String = String::new();

            if cfs_configuration["layers"].as_array().is_some() {
                let layers_json = cfs_configuration["layers"].as_array().unwrap();

                layers = format!(
                    "COMMIT: {} NAME: {}",
                    layers_json[0]["commit"], layers_json[0]["name"]
                );

                for layer in layers_json.iter().skip(1) {
                    layers = format!(
                        "{}\nCOMMIT: {} NAME: {}",
                        layers, layer["commit"], layer["name"]
                    );
                }
            }

            table.add_row(vec![
                cfs_configuration["name"].as_str().unwrap(),
                cfs_configuration["lastUpdated"].as_str().unwrap(),
                &layers,
            ]);
        }

        println!("{table}");
    }

    pub async fn create_from_repos(
        gitea_token: &str,
        gitea_base_url: &str,
        shasta_token: &str,
        shasta_base_url: &str,
        repos: Vec<String>,
        cfs_configuration_name_formatted: &String,
    ) -> configuration::CfsConfiguration {
        // Create CFS configuration
        let mut cfs_configuration = configuration::CfsConfiguration::new();

        for i in 0..repos.len() {
            // Get repo from path
            let repo = match local_git_repo::get_repo(repos.get(i).unwrap()) {
                Ok(repo) => repo,
                Err(_) => {
                    log::error!("Could not find a git repo in {}", repos[i]);
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

            let mut api_url = "cray/".to_string();
            api_url.push_str(repo_name);

            // Check if repo and local commit id exists in Shasta cvs
            let shasta_commitid_details_resp = gitea::http_client::get_commit_details(
                &api_url,
                // &format!("/cray/{}", repo_name),
                &local_last_commit.id().to_string(),
                &gitea_token,
            )
            .await;

            // Check sync status between user face and shasta VCS
            let shasta_commitid_details: Value = match shasta_commitid_details_resp {
                Ok(_) => {
                    log::debug!(
                        "Local latest commit id {} for repo {} exists in shasta",
                        local_last_commit.id(),
                        repo_name
                    );
                    shasta_commitid_details_resp.unwrap()
                }
                Err(e) => {
                    log::error!("{}", e);
                    std::process::exit(1);
                }
            };

            let mut clone_url = gitea_base_url.clone().to_string();
            clone_url.push_str("/cray/");
            clone_url.push_str(repo_name);

            // Create CFS layer
            let cfs_layer = configuration::Layer::new(
                clone_url,
                Some(String::from(
                    shasta_commitid_details["sha"].as_str().unwrap(),
                )),
                format!(
                    "{}-{}",
                    repo_name.substring(0, repo_name.len()),
                    chrono::offset::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                ),
                String::from("site.yml"),
                None,
            );

            cfs_configuration = configuration::add_layer(cfs_layer, cfs_configuration);
        }

        log::info!("CFS configuration:\n{:#?}", cfs_configuration);

        // Update/PUT CFS configuration
        log::debug!("Create configuration and session name.");
        configuration::http_client::put(
            &shasta_token,
            &shasta_base_url,
            &cfs_configuration,
            &cfs_configuration_name_formatted,
        )
        .await;

        cfs_configuration
    }

    pub fn get_git_repo_url_for_layer(gitea_base_url: &String, repo_name: &String) -> String {
        let mut clone_url = gitea_base_url.clone().to_string();
        clone_url.push_str("/cray/");
        clone_url.push_str(repo_name);

        clone_url
    }
}
