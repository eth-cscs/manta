use std::fmt;
use comfy_table::Table;

pub struct Configuration {
    name: String,
    last_updated: String,
    config_layers: Vec<Layer>
}

impl Configuration {
    pub fn new(name: &str, last_updated: &str, config_layers: Vec<Layer>) -> Self {
        Configuration {
            name: String::from(name),
            last_updated: String::from(last_updated),
            config_layers
        }
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "\nConfig Details:\n - name: {}\n - last updated: {}\nLayers:", self.name, self.last_updated)?;
        let mut cont = 0;

        for config_layer in &self.config_layers {

            write!(f, "\n Layer {}:{}", cont, config_layer)?;
            cont += 1;
        }

        Ok(())
    }
}

pub struct Layer {
    pub name: String,
    pub repo_name: String,
    pub commit_id: String,
    pub author: String,
    pub commit_date: String
}

impl Layer {
    pub fn new(name: &str, repo_name: &str, commit_id: &str, author: &str, commit_date: &str) -> Self {
        Layer {
            name: String::from(name),
            repo_name: String::from(repo_name),
            commit_id: String::from(commit_id),
            author: String::from(author),
            commit_date: String::from(commit_date)
        }
    }
}
    
impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n - name: {}\n - repo name: {}\n - commit id: {}\n - commit date: {}\n - author: {}", self.name, self.repo_name, self.commit_id, self.commit_date, self.author)
    }
}

pub fn print_table(cfs_configuration: Configuration) {
    
    let mut table = Table::new();

    table.set_header(vec!["Name", "Last updated", "Layers"]);

    let mut layers: String = String::new();

    if !cfs_configuration.config_layers.is_empty() {

        layers = format!("COMMIT: {} DATE: {} NAME: {} AUTHOR: {}", cfs_configuration.config_layers[0].commit_id, cfs_configuration.config_layers[0].commit_date, cfs_configuration.config_layers[0].name, cfs_configuration.config_layers[0].author);
        
        for i in 1..cfs_configuration.config_layers.len() {
            
            let layer = &cfs_configuration.config_layers[i];
            layers = format!("{}\nCOMMIT: {} DATE: {} NAME: {} AUTHOR: {}", layers, layer.commit_id, layer.commit_date, layer.name, layer.author);
        }
    }

    table.add_row(vec![
        cfs_configuration.name,
        cfs_configuration.last_updated,
        layers
    ]);

    println!("{table}");
}
