use comfy_table::Table;
use mesa::shasta;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Ansible {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passthrough: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Target {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<Group>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Artifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    #[serde(rename = "completionTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<String>,
    #[serde(rename = "startTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actifacts: Option<Vec<Artifact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Session>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CfsSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Configuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansible: Option<Ansible>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Target>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /* #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<...> */
}

pub fn get_cfs_session_list(cfs_session: Value) -> Vec<String> {
    let mut result = vec![cfs_session["name"].as_str().unwrap_or_default().to_string()];
    result.push(
        cfs_session
            .pointer("/configuration/name")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/status/session/startTime")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/ansible/passthrough")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/ansible/verbosity")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/status/session/status")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/status/session/succeeded")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/target/definition")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    let target = if let Some(groups) = cfs_session.pointer("/target/groups").unwrap().as_array() {
        groups
            .iter()
            .map(|group| group["name"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        cfs_session
            .pointer("/ansible/limit")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
            .replace(',', "\n")
    };
    result.push(target);
    /*     result.push(
        cfs_session
            .pointer("/target/groups")
            .unwrap()
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|group| group["name"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>()
            .join(",\n"),
    ); */
    result.push(
        cfs_session
            .pointer("/status/artifacts")
            .unwrap_or(&serde_json::Value::Array(Vec::new()))
            .as_array()
            .unwrap()
            .first()
            .unwrap_or(&serde_json::Value::String("".to_string()))["result_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );

    result
}

pub fn print_table(get_cfs_session_value_list: &Vec<Value>) {
    let mut table = Table::new();

    table.set_header(vec![
        "Name",
        "Configuration",
        "Start",
        "Passthrough",
        "Verbosity",
        "Status",
        "Succeeded",
        "Target Def",
        "Target",
        "Image ID",
    ]);

    for cfs_session_value in get_cfs_session_value_list {
        table.add_row(get_cfs_session_list(cfs_session_value.clone()));
    }

    println!("{table}");
}

pub async fn get_image_id_related_to_cfs_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: &String,
) -> Option<String> {
    // Get all CFS sessions which has succeeded
    let cfs_sessions_value_list = mesa::shasta::cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    get_image_id_from_cfs_session_list(
        shasta_token,
        shasta_base_url,
        cfs_configuration_name,
        &cfs_sessions_value_list,
    )
    .await
}

pub async fn get_image_id_from_cfs_session_list(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_configuration_name: &String,
    cfs_sessions_value_list: &[Value],
) -> Option<String> {
    // Filter CFS sessions to the ones related to CFS configuration and built an image (target
    // definition is 'image' and it actually has at least one artifact)
    let cfs_session_value_target_list =
        cfs_sessions_value_list.iter().filter(|cfs_session_value| {
            cfs_session_value
                .pointer("/configuration/name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
                .eq(cfs_configuration_name)
                && cfs_session_value
                    .pointer("/target/definition")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq("image")
                && cfs_session_value.pointer("/status/artifacts/0").is_some()
        });

    log::debug!(
        "All CFS sessions related to CFS configuration {}:\n{:#?}",
        cfs_configuration_name,
        cfs_session_value_target_list
    );

    // Find image in CFS sessions
    for cfs_session_value_target in cfs_session_value_target_list {
        log::debug!("CFS session details:\n{:#?}", cfs_session_value_target);

        let cfs_session_name = cfs_session_value_target["name"].as_str().unwrap();

        let image_id = cfs_session_value_target
            .pointer("/status/artifacts/0/result_id")
            .unwrap()
            .as_str();

        log::info!(
            "Checking image ID {} in CFS session {} exists",
            image_id.unwrap(),
            cfs_session_name
        );

        // Get IMS image related to the CFS session
        if mesa::shasta::ims::image::http_client::get(
            shasta_token,
            shasta_base_url,
            None,
            image_id,
            None,
        )
        .await
        .is_ok()
        {
            log::info!(
                "Image ID found related to CFS sesison {} is {}",
                cfs_session_name,
                image_id.unwrap()
            );

            return image_id.map(String::from); // from https://users.rust-lang.org/t/convert-option-str-to-option-string/20533/2
        };
    }

    None
}

pub async fn transform(
    shasta_token: &str,
    shasta_base_url: &str,
    cfs_sessions_value_list: Vec<Value>,
) -> Vec<Vec<String>> {
    let bos_sessiontemplate_list =
        shasta::bos::template::http_client::get(shasta_token, shasta_base_url, None, None, None)
            .await
            .unwrap();

    let mut cfs_session_table_data_list = Vec::new();

    for cfs_session in cfs_sessions_value_list {
        let mut cfs_session_table_data = Vec::new();
        let cfs_sesion_name = cfs_session["name"].as_str().unwrap();
        cfs_session_table_data.push(cfs_sesion_name.to_owned());
        let cfs_session_configuration_name = cfs_session["configuration"]["name"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_configuration_name.to_owned());
        let cfs_session_target_definition = cfs_session["target"]["definition"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_target_definition.to_owned());
        let cfs_session_target_groups = if cfs_session["target"]["groups"].as_array().is_some()
            && (cfs_session["target"]["groups"]
                .as_array()
                .unwrap()
                .iter()
                .len()
                > 0)
        {
            let cfs_session_target_groups_json =
                cfs_session["target"]["groups"].as_array().unwrap();

            let mut cfs_session_target_groups_aux =
                String::from(cfs_session_target_groups_json[0]["name"].as_str().unwrap());

            for (i, _) in cfs_session_target_groups_json.iter().enumerate().skip(1) {
                if i % 2 == 0 {
                    // breaking the cell content into multiple lines (only 2 target groups per line)
                    cfs_session_target_groups_aux.push_str(",\n");
                    // target_groups = format!("{},\n", target_groups);
                } else {
                    cfs_session_target_groups_aux.push_str(", ");
                    // target_groups = format!("{}, ", target_groups);
                }

                cfs_session_target_groups_aux
                    .push_str(cfs_session_target_groups_json[i]["name"].as_str().unwrap());
            }

            cfs_session_target_groups_aux
        } else {
            "".to_string()
        };

        let mut cfs_session_ansible_limit = cfs_session["ansible"]["limit"]
            .as_str()
            .unwrap_or_default()
            .split(',')
            .map(|xname| xname.trim());

        let first = cfs_session_ansible_limit.next();

        let cfs_session_ansible_limit = if let Some(first_xname) = first {
            let mut cfs_session_ansible_limit_aux = String::from(first_xname);

            let mut i = 1;

            for cfs_session_ansible_limit in cfs_session_ansible_limit {
                if i % 2 == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)
                    cfs_session_ansible_limit_aux.push_str(", \n");
                    // ansible_limits = format!("{},\n", ansible_limits);
                } else {
                    cfs_session_ansible_limit_aux.push_str(", ");
                    // ansible_limits = format!("{}, ", ansible_limits);
                }

                cfs_session_ansible_limit_aux.push_str(cfs_session_ansible_limit);
                // ansible_limits = format!("{}{}", ansible_limits, ansible_limit);

                i += 1;
            }

            cfs_session_ansible_limit_aux
        } else {
            "".to_string()
        };

        let cfs_session_target = if !cfs_session_target_groups.is_empty() {
            &cfs_session_target_groups
        } else {
            &cfs_session_ansible_limit
        };
        cfs_session_table_data.push(cfs_session_target.to_string());
        let cfs_session_status_session_starttime = cfs_session["status"]["session"]["startTime"]
            .as_str()
            .unwrap();
        cfs_session_table_data.push(cfs_session_status_session_starttime.to_string());

        let cfs_session_status_session_status =
            cfs_session["status"]["session"]["status"].as_str().unwrap();
        cfs_session_table_data.push(cfs_session_status_session_status.to_string());

        let cfs_session_status_session_succeeded = cfs_session["status"]["session"]["succeeded"]
            .as_str()
            .unwrap();
        cfs_session_table_data.push(cfs_session_status_session_succeeded.to_string());

        let cfs_session_status_artifacts_result_id = if !cfs_session["status"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            cfs_session["status"]["artifacts"][0]["result_id"]
                .as_str()
                .unwrap()
        } else {
            ""
        };

        // println!("{:#?}", cfs_session);

        let mut image_id_from_bos_sessiontemplate = "";
        if !cfs_session_status_artifacts_result_id.is_empty() {
            let bos_sessiontemplate =
                bos_sessiontemplate_list
                    .iter()
                    .find(|bos_session_template| {
                        bos_session_template
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .eq(cfs_session_configuration_name)
                    });
            if bos_sessiontemplate.is_some() {
                for (_boot_sets_param, boot_sets_value) in bos_sessiontemplate.unwrap()["boot_sets"]
                    .as_object()
                    .unwrap()
                {
                    if boot_sets_value.get("path").is_some() {
                        image_id_from_bos_sessiontemplate = boot_sets_value["path"]
                            .as_str()
                            .unwrap()
                            .trim_start_matches("s3://boot-images/")
                            .trim_end_matches("/manifest.json");
                        break;
                    }
                }
            } else {
                image_id_from_bos_sessiontemplate = cfs_session_status_artifacts_result_id;
            }
        }
        cfs_session_table_data.push(image_id_from_bos_sessiontemplate.to_string());
        // println!("{:#?}", ims_image_kernel_path);
        // println!("Hey! {:?}", cfs_session_table_data);
        cfs_session_table_data_list.push(cfs_session_table_data);
    }

    cfs_session_table_data_list
}
