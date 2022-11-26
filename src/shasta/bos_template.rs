use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>
}

impl Default for Link {
    fn default() -> Self {
        Self {
            rel: None,
            href: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boot_ordinal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shutdown_ordinal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    type_prop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_roles_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rootfs_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rootfs_provider_passthrough: Option<String>
}

impl Default for Property1 {
    fn default() -> Self {
        Self {
            name: None,
            boot_ordinal: None,
            shutdown_ordinal: None,
            path: None,
            type_prop: None,
            etag: None,
            kernel_parameters: None,
            network: None,
            node_list: None,
            node_roles_groups: None,
            node_groups: None,
            rootfs_provider: None,
            rootfs_provider_passthrough: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boot_ordinal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shutdown_ordinal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    type_prop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_roles_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rootfs_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rootfs_provider_passthrough: Option<String>
}

impl Default for Property2 {
    fn default() -> Self {
        Self {
            name: None,
            boot_ordinal: None,
            shutdown_ordinal: None,
            path: None,
            type_prop: None,
            etag: None,
            kernel_parameters: None,
            network: None,
            node_list: None,
            node_roles_groups: None,
            node_groups: None,
            rootfs_provider: None,
            rootfs_provider_passthrough: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    property1: Option<Property1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    property2: Option<Property2>
}

impl Default for BootSet {
    fn default() -> Self {
        Self {
            property1: None,
            property2: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cfs {
    #[serde(skip_serializing_if = "Option::is_none")]
    clone_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    playbook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    configuration: Option<String>
}

impl Default for Cfs {
    fn default() -> Self {
        Self {
            clone_url: None,
            branch: None,
            commit: None,
            playbook: None,
            configuration: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BosTemplate {
    name: String,
    #[serde(rename = "templateUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    template_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cfs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cfs_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_cfs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cfs: Option<Cfs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boot_sets: Option<BootSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<Vec<Link>>
}

pub mod http_client {

    use serde_json::Value;

    // pub async fn post(shasta_token: &str, shasta_base_url: &str, bos_template: BosTemplate) -> core::result::Result<Value, Box<dyn std::error::Error>> {

    //     log::debug!("Bos template:\n{:#?}", bos_template);
        
    //     // // socks5 proxy
    //     // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

    //     // // rest client create new cfs sessions
    //     // let client = reqwest::Client::builder()
    //     //     .danger_accept_invalid_certs(true)
    //     //     .proxy(socks5proxy)
    //     //     .build()?;

    //     let client;

    //     let client_builder = reqwest::Client::builder()
    //         .danger_accept_invalid_certs(true);
    
    //     // Build client
    //     if std::env::var("SOCKS5").is_ok() {
            
    //         // socks5 proxy
    //         let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
    
    //         // rest client to authenticate
    //         client = client_builder.proxy(socks5proxy).build()?;
    //     } else {
    //         client = client_builder.build()?;
    //     }
    
    //     let resp = client
    //         .post(format!("{}{}", shasta_base_url, "/bos/v1/sessiontemplate"))
    //         .bearer_auth(shasta_token)
    //         .json(&bos_template)
    //         .send()
    //         .await?;

    //     if resp.status().is_success() {
    //         Ok(serde_json::from_str(&resp.text().await?)?)
    //     } else {
    //         Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
    //     }
    // }

    pub async fn get(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, bos_template_name: &Option<String>, limit_number: &Option<u8>) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

        let mut cluster_bos_tempalte: Vec<Value> = Vec::new();

        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // // rest client get cfs sessions
        // let client = reqwest::Client::builder()
        //     .danger_accept_invalid_certs(true)
        //     .proxy(socks5proxy)
        //     .build()?;

        let client;

        let client_builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);
    
        // Build client
        if std::env::var("SOCKS5").is_ok() {
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
    
            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }
        
        let resp = client
            .get(format!("{}{}", shasta_base_url, "/bos/v1/sessiontemplate"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value;

        if resp.status().is_success() {
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    
        if cluster_name.is_some() {
            for bos_template in json_response.as_array().unwrap() {
    
                if bos_template["name"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) // TODO: investigate why I need to use this ugly 'as_ref'
                {
                    cluster_bos_tempalte.push(bos_template.clone());
                }

                // cluster_bos_tempalte.sort_by(|a, b| a["status"]["session"]["startTime"].as_str().unwrap().cmp(&b["status"]["session"]["startTime"].as_str().unwrap()));

            }
        } else if bos_template_name.is_some() {
            for bos_template in json_response.as_array().unwrap() {
                if bos_template["name"]
                    .as_str()
                    .unwrap()
                    .eq(bos_template_name.as_ref().unwrap()) // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_bos_tempalte.push(bos_template.clone());
                }
            }
        } else { // Returning all results
            cluster_bos_tempalte = json_response.as_array().unwrap().clone();

            // cluster_bos_tempalte.sort_by(|a, b| a["status"]["session"]["startTime"].as_str().unwrap().cmp(&b["status"]["session"]["startTime"].as_str().unwrap()));
        }
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            // cluster_cfs_sessions = json_response.as_array().unwrap().to_vec();
    
            // cluster_bos_tempalte.sort_by(|a, b| a["status"]["session"]["startTime"].as_str().unwrap().cmp(&b["status"]["session"]["startTime"].as_str().unwrap()));
    
            // cfs_utils::print_cfs_configurations(&cfs_configurations);
            
            // cluster_cfs_sessions.truncate(limit_number.unwrap().into());
            cluster_bos_tempalte = cluster_bos_tempalte[cluster_bos_tempalte.len().saturating_sub(limit_number.unwrap().into())..].to_vec();
            
            // cluster_cfs_sessions = vec![cluster_cfs_sessions]; // vec! macro for vector initialization!!! https://doc.rust-lang.org/std/vec/struct.Vec.html
        } 

        Ok(cluster_bos_tempalte)
    }
}

pub mod utils {

    use comfy_table::Table;
    use serde_json::Value;

    pub fn print_table(bos_templates: Vec<Value>) {
        
        let mut table = Table::new();

        table.set_header(vec!["Name", "Cfs configuration", "Cfs enabled","Compute Node groups", "Compute Etag", "Compute Path", "UAN Node groups", "UAN Etag", "UAN Path"]);
    
        for bos_template in bos_templates {

            let mut compute_target_groups = String::new();
            let mut uan_target_groups = String::new();

            if bos_template["boot_sets"].get("uan").is_some() {

                let uan_node_groups_json = bos_template["boot_sets"]["uan"]["node_groups"].as_array().unwrap();

                uan_target_groups = String::from(uan_node_groups_json[0].as_str().unwrap());

                for i in 1..uan_node_groups_json.len() {

                    if i % 2 == 0 { // breaking the cell content into multiple lines (only 2 target groups per line)
                        uan_target_groups = format!("{},\n", uan_target_groups);
                    } else {
                        uan_target_groups = format!("{}, ", uan_target_groups);
                    }
                    
                    uan_target_groups = format!("{}{}", uan_target_groups, uan_node_groups_json[i].as_str().unwrap());
                }
            }

            if bos_template["boot_sets"].get("compute").is_some() {
                
                let compute_node_groups_json = bos_template["boot_sets"]["compute"]["node_groups"].as_array().unwrap();

                compute_target_groups = String::from(compute_node_groups_json[0].as_str().unwrap());

                for i in 1..compute_node_groups_json.len() {

                    if i % 2 == 0 { // breaking the cell content into multiple lines (only 2 target groups per line)
                        compute_target_groups = format!("{},\n", compute_target_groups);
                    } else {
                        compute_target_groups = format!("{}, ", compute_target_groups);
                    }
                    
                    compute_target_groups = format!("{}{}", compute_target_groups, compute_node_groups_json[i].as_str().unwrap());
                }
            }

            table.add_row(vec![
                bos_template["name"].as_str().unwrap(),
                bos_template["cfs"]["configuration"].as_str().unwrap(),
                &bos_template["enable_cfs"].as_bool().unwrap().to_string(),
                &compute_target_groups,
                bos_template["boot_sets"]["compute"]["etag"].as_str().unwrap_or_default(),
                bos_template["boot_sets"]["compute"]["path"].as_str().unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["node_groups"].as_str().unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["etag"].as_str().unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["path"].as_str().unwrap_or_default()
            ]);
        }
    
        println!("{table}");
    }
}