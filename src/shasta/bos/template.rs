use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
}

// impl Default for Link {
//     fn default() -> Self {
//         Self {
//             rel: None,
//             href: None
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Property {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_ordinal: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shutdown_ordinal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_prop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_roles_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider_passthrough: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
    rootfs_provider_passthrough: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BootSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute: Option<Property>,
    /* #[serde(skip_serializing_if = "Option::is_none")]
    property2: Option<Property2>, */
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cfs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BosTemplate {
    pub name: String,
    #[serde(rename = "templateUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfs_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_cfs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfs: Option<Cfs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_sets: Option<BootSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

impl BosTemplate {
    /* pub fn from_sat_file_serde_yaml(bos_template_yaml: &serde_yaml::Value) -> Self {

        BosTemplate
    } */

    pub fn new_for_node_list(
        cfs_configuration_name: String,
        bos_session_template_name: String,
        ims_image_name: String,
        ims_image_path: String,
        ims_image_type: String,
        ims_image_etag: String,
        limit: Vec<String>,
    ) -> Self {
        let cfs = crate::shasta::bos::template::Cfs {
            clone_url: None,
            branch: None,
            commit: None,
            playbook: None,
            configuration: Some(cfs_configuration_name),
        };

        let compute_property = crate::shasta::bos::template::Property {
            name: Some(ims_image_name),
            boot_ordinal: Some(2),
            shutdown_ordinal: None,
            path: Some(ims_image_path),
            type_prop: Some(ims_image_type),
            etag: Some(ims_image_etag),
            kernel_parameters: Some(
                "ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}".to_string(),
            ),
            network: Some("nmn".to_string()),
            node_list: Some(limit),
            node_roles_groups: None,
            node_groups: None,
            rootfs_provider: Some("cpss3".to_string()),
            rootfs_provider_passthrough: Some("dvs:api-gw-service-nmn.local:300:nmn0".to_string()),
        };

        let boot_set = crate::shasta::bos::template::BootSet {
            compute: Some(compute_property),
        };

        crate::shasta::bos::template::BosTemplate {
            name: bos_session_template_name,
            template_url: None,
            description: None,
            cfs_url: None,
            cfs_branch: None,
            enable_cfs: Some(true),
            cfs: Some(cfs),
            partition: None,
            boot_sets: Some(boot_set),
            links: None,
        }
    }

    pub fn new_for_hsm_group(
        cfs_configuration_name: String,
        bos_session_template_name: String,
        ims_image_name: String,
        ims_image_path: String,
        ims_image_type: String,
        ims_image_etag: String,
        hsm_group: &String,
    ) -> Self {
        let cfs = crate::shasta::bos::template::Cfs {
            clone_url: None,
            branch: None,
            commit: None,
            playbook: None,
            configuration: Some(cfs_configuration_name),
        };

        let compute_property = crate::shasta::bos::template::Property {
            name: Some(ims_image_name),
            boot_ordinal: Some(2),
            shutdown_ordinal: None,
            path: Some(ims_image_path),
            type_prop: Some(ims_image_type),
            etag: Some(ims_image_etag),
            kernel_parameters: Some(
                "ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}".to_string(),
            ),
            network: Some("nmn".to_string()),
            node_list: None,
            node_roles_groups: None,
            node_groups: Some(vec![hsm_group.to_string()]),
            rootfs_provider: Some("cpss3".to_string()),
            rootfs_provider_passthrough: Some("dvs:api-gw-service-nmn.local:300:nmn0".to_string()),
        };

        let boot_set = crate::shasta::bos::template::BootSet {
            compute: Some(compute_property),
        };

        crate::shasta::bos::template::BosTemplate {
            name: bos_session_template_name,
            template_url: None,
            description: None,
            cfs_url: None,
            cfs_branch: None,
            enable_cfs: Some(true),
            cfs: Some(cfs),
            partition: None,
            boot_sets: Some(boot_set),
            links: None,
        }
    }
}

pub mod http_client {

    use serde_json::Value;

    use super::{utils::check_hsms_or_xnames_belongs_to_bos_sessiontemplate, BosTemplate};

    pub async fn post(
        shasta_token: &str,
        shasta_base_url: &str,
        bos_template: &BosTemplate,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Bos template:\n{:#?}", bos_template);

        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // // rest client create new cfs sessions
        // let client = reqwest::Client::builder()
        //     .danger_accept_invalid_certs(true)
        //     .proxy(socks5proxy)
        //     .build()?;

        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let resp = client
            .post(format!("{}{}", shasta_base_url, "/bos/v1/sessiontemplate"))
            .bearer_auth(shasta_token)
            .json(&bos_template)
            .send()
            .await?;

        if resp.status().is_success() {
            let response = &resp.text().await?;
            Ok(serde_json::from_str(response)?)
        } else {
            eprintln!("FAIL request: {:#?}", resp);
            let response: String = resp.text().await?;
            eprintln!("FAIL response: {:#?}", response);
            Err(response.into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    /// Get BOS session templates. Ref --> https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/
    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: Option<&String>,
        bos_template_name: Option<&String>,
        limit_number: Option<&u8>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        // println!("Get BOS sessiontemplate from HSM group {:?}", hsm_group_name);

        let mut cluster_bos_tempalte: Vec<Value> = Vec::new();

        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let api_url = shasta_base_url.to_owned() + "/bos/v1/sessiontemplate";

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/bos/v1/sessiontemplate"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value = if resp.status().is_success() {
            serde_json::from_str(&resp.text().await?)?
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        };

        if hsm_group_name.is_some() {
            println!("json_response for HSM {}", hsm_group_name.unwrap());
            for bos_template in json_response.as_array().unwrap() {
                for (_key, value) in bos_template["boot_sets"].as_object().unwrap() {
                    if value["node_groups"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .any(|node_group| node_group.eq(hsm_group_name.unwrap()))
                    {
                        println!("BOS sessiontemplate:\n{:#?}", bos_template);
                        cluster_bos_tempalte.push(bos_template.clone());
                    }
                }
            }
        } else if bos_template_name.is_some() {
            for bos_template in json_response.as_array().unwrap() {
                if bos_template["name"]
                    .as_str()
                    .unwrap()
                    .eq(bos_template_name.unwrap())
                // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_bos_tempalte.push(bos_template.clone());
                }
            }
        } else {
            // Returning all results
            cluster_bos_tempalte = json_response.as_array().unwrap().clone();
        }

        if limit_number.is_some() {
            // Limiting the number of results to return to client

            cluster_bos_tempalte = cluster_bos_tempalte[cluster_bos_tempalte
                .len()
                .saturating_sub(*limit_number.unwrap() as usize)..]
                .to_vec();
        }

        Ok(cluster_bos_tempalte)
    }

    /// Get BOS session templates. Ref --> https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/
    /// It filters by boot_sets.<property>.node_list containing nodes param or
    /// boot_sets.<property>.node_groups containing hsm_groups_names param
    pub async fn get_for_multiple_hsm_groups(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_groups_names: Vec<String>,
        nodes: Vec<String>,
        bos_template_name: Option<&String>,
        limit_number: Option<&u8>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut cluster_bos_tempalte: Vec<Value> = Vec::new();

        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let api_url = shasta_base_url.to_owned() + "/bos/v1/sessiontemplate";

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/bos/v1/sessiontemplate"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value = if resp.status().is_success() {
            serde_json::from_str(&resp.text().await?)?
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        };

        if !hsm_groups_names.is_empty() {
            for bos_template in json_response.as_array().unwrap() {
                if check_hsms_or_xnames_belongs_to_bos_sessiontemplate(
                    bos_template,
                    hsm_groups_names.clone(),
                    nodes.clone(),
                ) {
                    cluster_bos_tempalte.push(bos_template.clone());
                }
            }
        } else if bos_template_name.is_some() {
            for bos_template in json_response.as_array().unwrap() {
                if bos_template["name"]
                    .as_str()
                    .unwrap()
                    .eq(bos_template_name.unwrap())
                // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_bos_tempalte.push(bos_template.clone());
                }
            }
        } else {
            // Returning all results

            cluster_bos_tempalte = json_response.as_array().unwrap().clone();
        }

        if limit_number.is_some() {
            // Limiting the number of results to return to client

            cluster_bos_tempalte = cluster_bos_tempalte[cluster_bos_tempalte
                .len()
                .saturating_sub(*limit_number.unwrap() as usize)..]
                .to_vec();
        }

        Ok(cluster_bos_tempalte)
    }
}

pub mod utils {

    use comfy_table::Table;
    use serde_json::Value;

    pub fn check_hsms_or_xnames_belongs_to_bos_sessiontemplate(
        bos_sessiontemplate: &Value,
        hsm_groups_names: Vec<String>,
        xnames: Vec<String>,
    ) -> bool {
        let boot_set_type = if bos_sessiontemplate.pointer("/boot_sets/uan").is_some() {
            "uan"
        } else {
            "compute"
        };

        let empty_array_value = &serde_json::Value::Array(Vec::new());

        let bos_template_node_list = bos_sessiontemplate
            .pointer(&("/boot_sets/".to_owned() + boot_set_type + "/node_list"))
            .unwrap_or(empty_array_value)
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node.as_str().unwrap().to_string());

        for bos_template_node in bos_template_node_list {
            if xnames.contains(&bos_template_node) {
                return true;
            }
        }

        let bos_template_node_groups = bos_sessiontemplate
            .pointer(&("/boot_sets/".to_owned() + boot_set_type + "/node_list"))
            .unwrap_or(empty_array_value)
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node.as_str().unwrap().to_string());

        for bos_template_node in bos_template_node_groups {
            if hsm_groups_names.contains(&bos_template_node) {
                return true;
            }
        }

        false
    }

    pub fn print_table(bos_templates: Vec<Value>) {
        let mut table = Table::new();

        table.set_header(vec![
            "Name",
            "Cfs configuration",
            "Cfs enabled",
            "Compute Node groups",
            "Compute Etag",
            "Compute Path",
            "UAN Node groups",
            "UAN Etag",
            "UAN Path",
        ]);

        for bos_template in bos_templates {
            let mut compute_target_groups = String::new();
            let mut uan_target_groups;

            if bos_template["boot_sets"].get("uan").is_some() {
                let uan_node_groups_json =
                    bos_template["boot_sets"]["uan"]["node_groups"].as_array();

                if let Some(uan_node_groups_json_aux) = uan_node_groups_json {
                    uan_target_groups = String::from(uan_node_groups_json_aux[0].as_str().unwrap());
                } else {
                    uan_target_groups = "".to_string();
                }

                for (i, _) in uan_node_groups_json.iter().enumerate().skip(1) {
                    if i % 2 == 0 {
                        // breaking the cell content into multiple lines (only 2 target groups per line)
                        uan_target_groups.push_str(",\n");
                        // uan_target_groups = format!("{},\n", uan_target_groups);
                    } else {
                        uan_target_groups.push_str(", ");
                        // uan_target_groups = format!("{}, ", uan_target_groups);
                    }

                    uan_target_groups.push_str(uan_node_groups_json.unwrap()[i].as_str().unwrap());

                    // uan_target_groups = format!("{}{}", uan_target_groups, uan_node_groups_json[i].as_str().unwrap());
                }
            }

            if bos_template["boot_sets"].get("compute").is_some() {
                let compute_node_groups_json =
                    bos_template["boot_sets"]["compute"]["node_groups"].as_array();

                if let Some(compute_node_groups_json_aux) = compute_node_groups_json {
                    compute_target_groups =
                        String::from(compute_node_groups_json_aux[0].as_str().unwrap());
                } else {
                    compute_target_groups = "".to_string();
                }

                for (i, _) in compute_node_groups_json.iter().enumerate().skip(1) {
                    if i % 2 == 0 {
                        // breaking the cell content into multiple lines (only 2 target groups per line)

                        compute_target_groups.push_str(",\n");

                        // compute_target_groups = format!("{},\n", compute_target_groups);
                    } else {
                        compute_target_groups.push_str(", ");

                        // compute_target_groups = format!("{}, ", compute_target_groups);
                    }

                    compute_target_groups
                        .push_str(compute_node_groups_json.unwrap()[i].as_str().unwrap());

                    // compute_target_groups = format!("{}{}", compute_target_groups, compute_node_groups_json[i].as_str().unwrap());
                }
            }

            table.add_row(vec![
                bos_template["name"].as_str().unwrap(),
                bos_template["cfs"]["configuration"].as_str().unwrap(),
                &bos_template["enable_cfs"].as_bool().unwrap().to_string(),
                &compute_target_groups,
                bos_template["boot_sets"]["compute"]["etag"]
                    .as_str()
                    .unwrap_or_default(),
                bos_template["boot_sets"]["compute"]["path"]
                    .as_str()
                    .unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["node_groups"]
                    .as_str()
                    .unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["etag"]
                    .as_str()
                    .unwrap_or_default(),
                bos_template["boot_sets"]["uan"]["path"]
                    .as_str()
                    .unwrap_or_default(),
            ]);
        }

        println!("{table}");
    }
}
