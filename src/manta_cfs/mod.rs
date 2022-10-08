pub mod configuration {

    use std::fmt;
    use serde_json::Value;
    use super::layer;

    use crate::shasta_vcs_utils;

    pub struct Config {
        name: String,
        last_updated: String,
        config_layers: Vec<layer::ConfigLayer>
    }

    impl fmt::Display for Config {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

            write!(f, "\nConfig Details:\n - name: {}\n - last updated: {}\nLayers:", self.name, self.last_updated)?;
            let mut cont = 0;
            for config_layer in &self.config_layers {
                write!(f, "\n Layer {}:{}", cont, config_layer)?;
                cont = cont + 1;
            }

            Ok(())
        }
    }

    pub async fn create(shasta_config_details: &Value, gitea_token: &str) -> core::result::Result<Config, Box<dyn std::error::Error>> {

        // Convert layers
        let mut config_layers: Vec<layer::ConfigLayer> = vec![];
        for layer in shasta_config_details["layers"].as_array().unwrap() {
            // Get CFS layer details from Gitea
            let gitea_commit_details = shasta_vcs_utils::http_client::get_commit_details(layer["cloneUrl"].as_str().unwrap(), layer["commit"].as_str().unwrap(), &gitea_token).await?;
            config_layers.push(layer::create(layer, gitea_commit_details).unwrap());
        }

        // Instantiate image layer struct
        let config = Config {
            name: String::from(shasta_config_details["name"].as_str().unwrap()),
            last_updated: String::from(shasta_config_details["lastUpdated"].as_str().unwrap()),
            config_layers: config_layers
        };

        Ok(config)
    }
}

pub mod layer {

    use std::fmt;
    use serde_json::Value;

    pub struct ConfigLayer {
        pub name: String,
        pub repo_name: String,
        pub commit_id: String,
        pub author: String,
        pub commit_date: String
    }
        
    impl fmt::Display for ConfigLayer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\n - name: {}\n - repo name: {}\n - commit id: {}\n - commit date: {}\n - author: {}", self.name, self.repo_name, self.commit_id, self.commit_date, self.author)
        }
    }
    
    pub fn create(layer: &Value, gitea_commit_details: Value) -> core::result::Result<ConfigLayer, Box<dyn std::error::Error>> { // TODO: convert this to constructior 'imp' *I think*
            
        // Instantiate image layer struct
        let image_layer = ConfigLayer {
            name: String::from(layer["name"].as_str().unwrap()),
            repo_name: String::from(layer["cloneUrl"].as_str().unwrap().trim_start_matches("https://api-gw-service-nmn.local/vcs/").trim_end_matches(".git")),
            commit_id: String::from(layer["commit"].as_str().unwrap()),
            author: String::from(gitea_commit_details["commit"]["committer"]["name"].as_str().unwrap()),
            commit_date: String::from(gitea_commit_details["commit"]["committer"]["date"].as_str().unwrap())
        };
    
        Ok(image_layer)
    }
}