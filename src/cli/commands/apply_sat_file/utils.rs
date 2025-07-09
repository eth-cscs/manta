use std::collections::{BTreeMap, HashMap};

use csm_rs::{
    bos::{
        self,
        session::shasta::http_client::v2::{BosSession, Operation},
        template::csm_rs::r#struct::v2::{BootSet, BosSessionTemplate, Cfs},
    },
    cfs::{
        self,
        configuration::csm_rs::r#struct::{
            cfs_configuration_request::v2::CfsConfigurationRequest,
            cfs_configuration_response::v2::CfsConfigurationResponse,
        },
        session::csm_rs::r#struct::v2::CfsSessionPostRequest,
    },
    common::jwt_ops,
    error::Error,
    hsm, ims,
};
use image::Image;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_yaml::{Mapping, Value};
use uuid::Uuid;

use crate::{
    cli::process::validate_target_hsm_members,
    common::{audit::Audit, kafka::Kafka},
};

use self::sessiontemplate::SessionTemplate;

#[derive(Deserialize, Serialize, Debug)]
pub struct SatFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configurations: Option<Vec<configuration::Configuration>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<image::Image>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_templates: Option<Vec<sessiontemplate::SessionTemplate>>,
}

impl SatFile {
    /// Filter either images or session_templates section according to user request
    pub fn filter(&mut self, image_only: bool, session_template_only: bool) {
        // Clean SAT template file if user only wan'ts to process the 'images' section. In this case,
        // we will remove 'session_templates' section from SAT fiel and also the entries in
        // 'configurations' section not used
        if image_only {
            let image_vec_opt: Option<&Vec<Image>> = self.images.as_ref();

            let configuration_name_image_vec: Vec<String> = match image_vec_opt {
                Some(image_vec) => image_vec
                    .iter()
                    .filter_map(|sat_template_image| sat_template_image.configuration.clone())
                    .collect(),
                None => {
                    eprintln!("ERROR - 'images' section missing in SAT file");
                    std::process::exit(1);
                }
            };

            // Remove configurations not used by any image
            self.configurations
                .as_mut()
                .unwrap_or(&mut Vec::new())
                .retain(|configuration| configuration_name_image_vec.contains(&configuration.name));

            // Remove section "session_templates"
            self.session_templates = None;
        }

        // Clean SAT template file if user only wan'ts to process the 'session_template' section. In this case,
        // we will remove 'images' section from SAT fiel and also the entries in
        // 'configurations' section not used
        if session_template_only {
            let sessiontemplate_vec_opt: Option<&Vec<SessionTemplate>> =
                self.session_templates.as_ref();

            let configuration_name_sessiontemplate_vec: Vec<String> = match sessiontemplate_vec_opt
            {
                Some(sessiontemplate_vec) => sessiontemplate_vec
                    .iter()
                    .map(|sat_sessiontemplate| sat_sessiontemplate.configuration.clone())
                    .collect(),
                None => {
                    eprintln!("ERROR - 'session_templates' section not defined in SAT file");
                    std::process::exit(1);
                }
            };

            // Remove configurations not used by any sessiontemplate
            /* self.configurations
            .as_mut()
            .unwrap_or(&mut Vec::new())
            .retain(|configuration| {
                configuration_name_sessiontemplate_vec.contains(&configuration.name)
            }); */

            if let Some(&[_]) = self.configurations.as_deref() {
                self.configurations
                    .as_mut()
                    .unwrap_or(&mut Vec::new())
                    .retain(|configuration| {
                        configuration_name_sessiontemplate_vec.contains(&configuration.name)
                    })
            } else {
                self.configurations = None;
            }

            let image_name_sessiontemplate_vec: Vec<String> = self
                .session_templates
                .as_ref()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|sessiontemplate| match &sessiontemplate.image {
                    sessiontemplate::Image::ImageRef { image_ref } => Some(image_ref),
                    sessiontemplate::Image::Ims { ims } => match ims {
                        sessiontemplate::ImsDetails::Name { name } => Some(name),
                        sessiontemplate::ImsDetails::Id { .. } => None,
                    },
                })
                .cloned()
                .collect();

            // Remove images not used by any sessiontemplate
            self.images
                .as_mut()
                .unwrap_or(&mut Vec::new())
                .retain(|image| image_name_sessiontemplate_vec.contains(&image.name));

            if self.images.as_ref().is_some_and(|images| images.is_empty()) {
                self.images = None;
            }
        }
    }
}
/// struct to represent the `session_templates` section in SAT file
pub mod sessiontemplate {
    use std::collections::HashMap;
    use strum_macros::Display;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct SessionTemplate {
        pub name: String,
        pub image: Image,
        pub configuration: String,
        pub bos_parameters: BosParamters,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum ImsDetails {
        Name { name: String },
        Id { id: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Image {
        Ims { ims: ImsDetails },
        ImageRef { image_ref: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct BosParamters {
        pub boot_sets: HashMap<String, BootSet>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct BootSet {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arch: Option<Arch>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kernel_parameters: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub network: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_list: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_roles_group: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_groups: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rootfs_provider: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rootfs_provider_passthrough: Option<String>,
    }

    #[derive(Deserialize, Serialize, Debug, Display)]
    pub enum Arch {
        X86,
        ARM,
        Other,
        Unknown,
    }
}

/// Convert from `sessiontemplate` in SAT file to manta BosSessionTemplate
/// example from https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html
impl TryFrom<SessionTemplate> for BosSessionTemplate {
    type Error = ();

    fn try_from(value: SessionTemplate) -> Result<BosSessionTemplate, Self::Error> {
        let b_st_cfs = Cfs {
            configuration: Some(value.configuration),
        };

        let mut boot_set_map: HashMap<String, BootSet> = HashMap::new();

        for (property, boot_set) in value.bos_parameters.boot_sets {
            let boot_set = BootSet {
                name: Some(format!(
                    "Boot set property '{}' created by manta from SAT file",
                    property
                )),
                path: None,
                r#type: None,
                etag: None,
                kernel_parameters: None,
                /* node_list: boot_set.node_list.map(|node_list| {
                    node_list
                        .split(",")
                        .map(|value| value.to_string())
                        .collect::<Vec<String>>()
                }),
                node_roles_groups: boot_set.node_groups.clone().map(|node_roles_groups| {
                    node_roles_groups
                        .split(",")
                        .map(|value| value.to_string())
                        .collect::<Vec<String>>()
                }),
                node_groups: boot_set.node_groups.map(|node_group| {
                    node_group
                        .split(",")
                        .map(|value| value.to_string())
                        .collect::<Vec<String>>()
                }), */
                node_list: boot_set.node_list,
                node_roles_groups: boot_set.node_roles_group,
                node_groups: boot_set.node_groups,
                // rootfs_provider: Some("cpss3".to_string()),
                rootfs_provider: boot_set.rootfs_provider,
                rootfs_provider_passthrough: boot_set.rootfs_provider_passthrough,
                cfs: Some(b_st_cfs.clone()),
                arch: boot_set.arch.map(|value| value.to_string()),
            };

            boot_set_map.insert(property, boot_set);
        }

        let b_st = BosSessionTemplate {
            name: Some(value.name),
            description: Some(format!(
                "BOS sessiontemplate created by manta from SAT file"
            )),
            enable_cfs: Some(true),
            cfs: Some(b_st_cfs),
            boot_sets: Some(boot_set_map),
            links: None,
            tenant: None,
        };

        Ok(b_st)
    }
}

/// struct to represent the `images` section in SAT file
pub mod image {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub enum Arch {
        #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
        Aarch64,
        #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
        X86_64,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Ims {
        Name { name: String, r#type: String },
        Id { id: String, r#type: String },
        BackwardCompatible { is_recipe: Option<bool>, id: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Filter {
        Prefix { prefix: String },
        Wildcard { wildcard: String },
        Arch { arch: Arch },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Product {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        r#type: String,
        filter: Filter,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Base {
        Ims { ims: Ims },
        Product { product: Product },
        ImageRef { image_ref: String },
    }

    // Used for backguard compatibility
    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum BaseOrIms {
        Base { base: Base },
        Ims { ims: Ims },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Image {
        pub name: String,
        #[serde(flatten)]
        pub base_or_ims: BaseOrIms,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub configuration: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub configuration_group_names: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ref_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
    }
}

/// struct to represent the `configurations` section in SAT file
pub mod configuration {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Product {
        ProductVersionBranch {
            name: String,
            version: Option<String>,
            branch: String,
        },
        ProductVersionCommit {
            name: String,
            version: Option<String>,
            commit: String,
        },
        ProductVersion {
            name: String,
            version: String,
        },
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Git {
        GitCommit { url: String, commit: String },
        GitBranch { url: String, branch: String },
        GitTag { url: String, tag: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum LayerType {
        Git { git: Git },
        Product { product: Product },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Layer {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(default = "default_playbook")]
        pub playbook: String, // This field is optional but with default value. Therefore we won't
        #[serde(flatten)]
        pub layer_type: LayerType,
    }

    fn default_playbook() -> String {
        "site.yml".to_string()
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum Inventory {
        InventoryCommit {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            url: String,
            commit: String,
        },
        InventoryBranch {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            url: String,
            branch: String,
        },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Configuration {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub layers: Vec<Layer>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub additional_inventory: Option<Inventory>,
    }
}

pub mod sat_file_image_old {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Ims {
        is_recipe: bool,
        id: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Product {
        name: String,
        version: String,
        r#type: String,
    }

    /* pub struct SatFileImage {
        pub name: String,
        pub ims: Ims,
        pub configuration: Option<String>,
        pub configuration_group_names: Option<Vec<String>>,
        pub ref_name: Option<String>,
        pub description: Option<String>,
    } */
}

/// Merge 2 yamls, 'b' values will overwrite 'a' values
/// eg:
/// having a:
///
/// ```
/// key_1
///   key_1_1: value_1_1
///   key_1_2: value_1_2
/// key_2: value_2
/// key_3: value_3
/// ```
/// and b:
/// ```
/// key_1
///   key_1_1: new_value_1_1
///   key_1_2: value_1_2
///   key_1_3: new_value_1_3
/// key_2: new_value_2
/// key_4: new_value_4
/// ```
/// would convert a into:
/// ```
/// key_1
///   key_1_1: new_value_1_1
///   key_1_3: new_value_1_3
/// key_2: new_value_2
/// key_3: value_3
/// key_4: new_value_4
/// ```
fn merge_yaml(base: Value, merge: Value) -> Option<Value> {
    match (base, merge) {
        (Value::Mapping(mut base_map), Value::Mapping(merge_map)) => {
            for (key, value) in merge_map {
                if let Some(base_value) = base_map.get_mut(&key) {
                    *base_value = merge_yaml(base_value.clone(), value)?;
                } else {
                    base_map.insert(key, value);
                }
            }
            Some(Value::Mapping(base_map))
        }
        (Value::Sequence(mut base_seq), Value::Sequence(merge_seq)) => {
            base_seq.extend(merge_seq);
            Some(Value::Sequence(base_seq))
        }
        (_, merge) => Some(merge),
    }
}

/// Convert a String dot notation expression into a serde_yaml::Value.
/// eg:
/// dot notation input like:
/// ```
/// key_1.key_2.key_3=1
/// ````
/// would result in a serde_yaml::Value equivalent to:
/// ```
/// key_1
///   key_2
///     key_3: 1
/// ```
fn dot_notation_to_yaml(dot_notation: &str) -> Result<serde_yaml::Value, Error> {
    let parts: Vec<&str> = dot_notation.split('=').collect();
    if parts.len() != 2 {
        return Err(Error::Message("Invalid format".to_string()));
    }

    let keys: Vec<&str> = parts[0].trim().split('.').collect();
    let value_str = parts[1].trim().trim_matches('"'); // Remove leading and trailing quotes
    let value: Value = Value::String(value_str.to_string());

    let mut root = Value::Mapping(Mapping::new());
    let mut current_level = &mut root;

    for (i, &key) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            // Last key, assign the value
            if let Value::Mapping(map) = current_level {
                map.insert(Value::String(key.to_string()), value.clone());
            }
        } else {
            // Not the last key, create or use existing map
            let next_level = if let Value::Mapping(map) = current_level {
                if map.contains_key(&Value::String(key.to_string())) {
                    // Use existing map
                    map.get_mut(&Value::String(key.to_string())).unwrap()
                } else {
                    // Create new map and insert
                    map.insert(
                        Value::String(key.to_string()),
                        Value::Mapping(Mapping::new()),
                    );
                    map.get_mut(&Value::String(key.to_string())).unwrap()
                }
            } else {
                // In case the structure is not as expected; should not happen in this logic
                return Err(Error::Message(
                    "Unexpected structure encountered".to_string(),
                ));
            };
            current_level = next_level;
        }
    }

    Ok(root)
}

pub fn render_jinja2_sat_file_yaml(
    sat_file_content: &String,
    values_file_content_opt: Option<&String>,
    value_cli_vec_opt: Option<Vec<String>>,
) -> Value {
    let mut env = minijinja::Environment::new();
    // Set/enable debug in order to force minijinja to print debug error messages which are more
    // descriptive. Eg https://github.com/mitsuhiko/minijinja/blob/main/examples/error/src/main.rs#L4-L5
    env.set_debug(true);
    // Set lines starting with `#` as comments
    env.set_syntax(
        minijinja::syntax::SyntaxConfig::builder()
            .line_comment_prefix("#")
            .build()
            .unwrap(),
    );
    // Set 'String' as undefined behaviour meaning, missing values won't pass the template
    // rendering
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

    // Render session values file
    let mut values_file_yaml: Value = if let Some(values_file_content) = values_file_content_opt {
        log::info!("'Session vars' file provided. Going to process SAT file as a template.");
        // Read sesson vars file and parse it to YAML
        let values_file_yaml: Value = serde_yaml::from_str(values_file_content).unwrap();
        // Render session vars file with itself (copying ansible behaviour where the ansible vars
        // file is also a jinja template and combine both vars and values in it)
        let values_file_rendered = env
            .render_str(values_file_content, values_file_yaml)
            .expect("ERROR - Error parsing values file to YAML. Exit");
        serde_yaml::from_str(&values_file_rendered).unwrap()
    } else {
        serde_yaml::from_str(sat_file_content).unwrap()
    };

    // Convert variable values sent by cli argument from dot notation to yaml format
    log::debug!("Convert variable values sent by cli argument from dot notation to yaml format");
    if let Some(value_option_vec) = value_cli_vec_opt {
        for value_option in value_option_vec {
            let cli_var_context_yaml_rslt = dot_notation_to_yaml(&value_option);
            let cli_var_context_yaml = match cli_var_context_yaml_rslt {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("ERROR - {:#?}", e);
                    panic!();
                }
            };
            values_file_yaml = merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).unwrap();
        }
    }

    // render sat template file
    let sat_file_rendered_rslt = env.render_str(sat_file_content, values_file_yaml);

    let sat_file_rendered = match sat_file_rendered_rslt {
        Ok(sat_file_rendered) => sat_file_rendered,
        Err(err) => {
            eprintln!("ERROR - Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                eprintln!();
                eprintln!("caused by: {:#}", next_err);
                err = next_err;
            }

            std::process::exit(1);
        }
    };
    // Disable debug
    env.set_debug(false);

    let sat_file_yaml: Value = serde_yaml::from_str(&sat_file_rendered).unwrap();

    sat_file_yaml
}

pub async fn create_cfs_configuration_from_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    cray_product_catalog: &BTreeMap<String, String>,
    sat_file_configuration_yaml: &serde_yaml::Value,
    // tag: &str,
    site_name: &str,
    dry_run: bool,
) -> Result<CfsConfigurationResponse, Error> {
    log::debug!(
        "Convert CFS configuration in SAT file (yaml):\n{:#?}",
        sat_file_configuration_yaml
    );

    let (cfs_configuration_name, mut cfs_configuration) =
        CfsConfigurationRequest::from_sat_file_serde_yaml(
            shasta_root_cert,
            gitea_base_url,
            gitea_token,
            sat_file_configuration_yaml,
            cray_product_catalog,
            site_name,
        )
        .await;

    if !dry_run {
        cfs::configuration::csm_rs::utils::create(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_configuration_name,
            &mut cfs_configuration,
        )
        .await
        .map_err(|e| {
            Error::Message(format!(
                "Error creating CFS configuration '{}': {}",
                cfs_configuration_name, e
            ))
        })
    } else {
        println!("Create CFS configuration:\n{:#?}", cfs_configuration);

        let cfs_configuration = CfsConfigurationResponse {
            name: cfs_configuration_name,
            last_updated: "".to_string(),
            layers: Vec::new(),
            additional_inventory: None,
        };

        Ok(cfs_configuration)
    }
}

/// Analyze a list of images in SAT file and returns the image to process next.
/// Input values:
///  - image_yaml_vec: the list of images in the SAT file, each element is a serde_yaml::Value
///  - ref_name_processed_vec: he list of images (ref_name) already processed
/// Note:
/// image.base.image_ref value in SAT file points to the image it depends on (image.ref_name)
/// NOTE 2: we assume that there may be a mix of images in SAT file with and without "ref_name"
/// value, we will use the function "get_ref_name" which will fall back to "name" field if
/// "ref_name" is missing in the image
/// An image is ready to be processed if:
///  - It does not depends on another image (image.base.image_ref is missing)
///  - The image it depends to is already processed (image.base.image_ref included in
///  ref_name_processed)
///  - It has not been already processed
pub fn get_next_image_in_sat_file_to_process(
    image_yaml_vec: &[serde_yaml::Value],
    ref_name_processed_vec: &[String],
) -> Option<serde_yaml::Value> {
    image_yaml_vec
        .iter()
        .find(|image_yaml| {
            let ref_name: &str = &get_image_name_or_ref_name_to_process(image_yaml); // Again, because we assume images in
                                                                                     // SAT file may or may not have ref_name value, we will use "get_ref_name" function to
                                                                                     // get the id of the image

            let image_base_image_ref_opt: Option<&str> =
                image_yaml.get("base").and_then(|image_base_yaml| {
                    image_base_yaml
                        .get("image_ref")
                        .and_then(|image_base_image_ref_yaml| image_base_image_ref_yaml.as_str())
                });

            !ref_name_processed_vec.contains(&ref_name.to_string())
                && (image_base_image_ref_opt.is_none()
                    || image_base_image_ref_opt.is_some_and(|image_base_image_ref| {
                        ref_name_processed_vec.contains(&image_base_image_ref.to_string())
                    }))
        })
        .cloned()
}

/// Get the "ref_name" from an image, because we need to be aware of which images in SAT file have
/// been processed in order to find the next image to process. We assume not all images in the yaml
/// will have an "image_ref" value, therefore we will use "ref_name" or "name" field if the former
/// is missing
pub fn get_image_name_or_ref_name_to_process(image_yaml: &serde_yaml::Value) -> String {
    if image_yaml.get("ref_name").is_some() {
        image_yaml["ref_name"].as_str().unwrap().to_string()
    } else {
        // If the image processed is missing the field "ref_name", then use the field "name"
        // instead, this is needed to flag this image as processed and filtered when
        // calculating the next image to process (get_next_image_to_process)
        image_yaml["name"].as_str().unwrap().to_string()
    }
}

pub async fn import_images_section_in_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    ref_name_processed_hashmap: &mut HashMap<String, String>,
    image_yaml_vec: Vec<serde_yaml::Value>,
    cray_product_catalog: &BTreeMap<String, String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    debug_on_failure: bool, // tag: &str,
    dry_run: bool,
    watch_logs: bool,
) -> HashMap<String, serde_yaml::Value> {
    // Get an image to process (the image either has no dependency or it's image dependency has
    // already ben processed)
    let mut next_image_to_process_opt: Option<serde_yaml::Value> =
        get_next_image_in_sat_file_to_process(
            &image_yaml_vec,
            &ref_name_processed_hashmap
                .keys()
                .cloned()
                .collect::<Vec<String>>(),
        );

    // Process images
    let mut image_processed_hashmap: HashMap<String, serde_yaml::Value> = HashMap::new();

    while let Some(image_yaml) = &next_image_to_process_opt {
        let image_id = create_image_from_sat_file_serde_yaml(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            k8s_api_url,
            image_yaml,
            cray_product_catalog,
            ansible_verbosity_opt,
            ansible_passthrough_opt,
            ref_name_processed_hashmap,
            debug_on_failure,
            dry_run,
            watch_logs,
        )
        .await
        .unwrap();

        image_processed_hashmap.insert(image_id.clone(), image_yaml.clone());

        ref_name_processed_hashmap.insert(
            get_image_name_or_ref_name_to_process(image_yaml),
            image_id.clone(),
        );

        next_image_to_process_opt = get_next_image_in_sat_file_to_process(
            &image_yaml_vec,
            &ref_name_processed_hashmap
                .keys()
                .cloned()
                .collect::<Vec<String>>(),
        );
    }

    image_processed_hashmap
}

pub async fn create_image_from_sat_file_serde_yaml(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    image_yaml: &serde_yaml::Value, // NOTE: image may be an IMS job or a CFS session
    cray_product_catalog: &BTreeMap<String, String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    ref_name_image_id_hashmap: &HashMap<String, String>,
    debug_on_failure: bool,
    dry_run: bool,
    watch_logs: bool,
) -> Result<String, Error> {
    // Collect CFS session details from SAT file
    // Get CFS session name from SAT file
    let image_name = image_yaml["name"].as_str().unwrap().to_string();
    // let image_name = image_yaml["name"]
    //     .as_str()
    //     .unwrap()
    //     .to_string()
    //     .replace("__DATE__", tag);

    log::info!(
        "Creating CFS session related to build image '{}'",
        image_name
    );

    // Get CFS configuration related to CFS session in SAT file
    let configuration_name: String = image_yaml["configuration"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    // Rename session's configuration name
    // configuration = configuration.replace("__DATE__", tag);

    // Get HSM groups related to CFS session in SAT file
    let groups_name: Vec<String> = image_yaml["configuration_group_names"]
        .as_sequence()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|group_name| group_name.as_str().unwrap().to_string())
        .collect();

    // VALIDATION: make sure grups in SAT.images "CFS session" are valid
    // NOTE: this is temporary until we get rid off "group" names as ansible folder names
    let invalid_groups: Vec<String> =
        csm_rs::cfs::session::csm_rs::utils::validate_groups(&groups_name, shasta_token);

    if !invalid_groups.is_empty() {
        log::debug!("CFS session group validation - failed");

        return Err(Error::Message(format!(
            "Please fix 'images' section in SAT file.\nInvalid groups: {:?}",
            invalid_groups
        )));
    } else {
        log::debug!("CFS session group validation - passed");
    }

    let base_image_id: String;

    // Get/process base image
    if let Some(sat_file_image_ims_value_yaml) = image_yaml.get("ims") {
        // ----------- BASE IMAGE - BACKWARD COMPATIBILITY WITH PREVIOUS SAT FILE
        log::info!("SAT file - 'image.ims' job ('images' section in SAT file is outdated - switching to backward compatibility)");

        base_image_id = process_sat_file_image_old_version(sat_file_image_ims_value_yaml).unwrap();
    } else if let Some(sat_file_image_base_value_yaml) = image_yaml.get("base") {
        if let Some(sat_file_image_base_image_ref_value_yaml) =
            sat_file_image_base_value_yaml.get("image_ref")
        {
            log::info!("SAT file - 'image.base.image_ref' job");

            base_image_id = process_sat_file_image_ref_name(
                sat_file_image_base_image_ref_value_yaml,
                ref_name_image_id_hashmap,
            )
            .unwrap();
        } else if let Some(sat_file_image_base_ims_value_yaml) =
            sat_file_image_base_value_yaml.get("ims")
        {
            log::info!("SAT file - 'image.base.ims' job");
            let ims_job_type = sat_file_image_base_ims_value_yaml["type"].as_str().unwrap();
            if ims_job_type == "recipe" {
                log::info!("SAT file - 'image.base.ims' job of type 'recipe'");

                base_image_id = process_sat_file_image_ims_type_recipe(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    sat_file_image_base_ims_value_yaml,
                    &image_name,
                )
                .await
                .unwrap();
            } else if ims_job_type == "image" {
                log::info!("SAT file - 'image.base.ims' job of type 'image'");

                base_image_id = sat_file_image_base_ims_value_yaml["id"]
                    .as_str()
                    .unwrap()
                    .to_string();
            } else {
                return Err(Error::Message(
                    "Can't process SAT file 'images.base.ims' is missing. Exit".to_string(),
                ));
            }

        // ----------- BASE IMAGE - CRAY PRODUCT CATALOG
        } else if let Some(sat_file_image_base_product_value_yaml) =
            sat_file_image_base_value_yaml.get("product")
        {
            log::info!("SAT file - 'image.base.product' job");
            // Base image created from a cray product
            let product_name = sat_file_image_base_product_value_yaml["name"]
                .as_str()
                .unwrap();

            let product_version = sat_file_image_base_product_value_yaml["version"]
                .as_str()
                .unwrap();

            let product_type = sat_file_image_base_product_value_yaml["type"]
                .as_str()
                .unwrap()
                .to_string()
                + "s";

            // We assume the SAT file has been alredy validated therefore taking some risks in
            // getting the details from the Cray product catalog
            let product_image_map =
                &serde_yaml::from_str::<serde_json::Value>(&cray_product_catalog[product_name])
                    .unwrap()[product_version][product_type.clone()]
                .as_object()
                .unwrap()
                .clone();

            let image_id = if let Some(filter) =
                sat_file_image_base_product_value_yaml.get("filter")
            {
                filter_product_catalog_images(filter, product_image_map.clone(), &image_name)
                    .unwrap()
            } else {
                // There is no 'image.product.filter' value defined in SAT file. Check Cray
                // product catalog only has 1 image. Othewise fail
                log::info!("No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image");
                product_image_map
                    .values()
                    .next()
                    .and_then(|value| value.get("id"))
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            };

            // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE RECIPE
            base_image_id = if product_type == "recipes" {
                // Create base image from an IMS job (the 'id' field in
                // images[].base.product.id is the id of the IMS recipe used to
                // build the new base image)

                log::info!("SAT file - 'image.base.product' job based on IMS recipes");

                let product_recipe_id = image_id.clone();

                process_sat_file_image_product_type_ims_recipe(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &product_recipe_id,
                    &image_name,
                )
                .await
                .unwrap()

                // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE IMAGE
            } else if product_type == "images" {
                // Base image already created and its id is available in the Cray
                // product catalog

                log::info!("SAT file - 'image.base.product' job based on IMS images");

                log::info!("Getting base image id from Cray product catalog");

                let product_image_id = image_id;

                /* let product_image_id = product_image_map
                .as_object()
                .unwrap()
                .values()
                .collect::<Vec<_>>()
                .first()
                .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string(); */

                product_image_id
            } else {
                return Err(Error::Message(
                    "Can't process SAT file, field 'images.base.product.type' must be either 'images' or 'recipes'. Exit".to_string(),
                ));
            }
        } else {
            return Err(Error::Message(
                "Can't process SAT file 'images.base.product' is missing. Exit".to_string(),
            ));
        }
    } else {
        return Err(Error::Message(
            "Can't process SAT file 'images.base' is missing. Exit".to_string(),
        ));
    }

    if configuration_name.is_empty() {
        log::info!("No CFS session needs to be created since there is no CFS configuration assigned to this image");
        println!(
            "Image '{}' imported image_id '{}'",
            image_name, base_image_id
        );

        Ok(base_image_id)
    } else {
        // Create a CFS session
        log::info!("Creating CFS session");

        // Create CFS session
        let cfs_session = CfsSessionPostRequest::new(
            image_name.clone(),
            configuration_name,
            // None,
            None,
            // None,
            ansible_verbosity_opt,
            ansible_passthrough_opt.cloned(),
            true,
            Some(groups_name.to_vec()),
            Some(base_image_id),
            // None,
            // debug_on_failure,
            // Some(image_name.clone()),
        );

        if !dry_run {
            let cfs_session = cfs::session::csm_rs::http_client::post_sync(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                k8s_api_url,
                &cfs_session,
                watch_logs,
            )
            .await?;

            if !cfs_session.is_success() {
                eprintln!(
                    "Error: CFS session '{}' failed. Exit",
                    cfs_session.name.unwrap()
                );
                std::process::exit(1);
            }

            let image_id = cfs_session.get_first_result_id().unwrap();
            println!("Image '{}' ({}) created", image_name, image_id);

            Ok(image_id)
        } else {
            println!("Create CFS session:\n{:#?}", cfs_session);

            let image_id = Uuid::new_v4().to_string();

            println!("Image '{}' ({}) created", image_name, image_id);

            Ok(image_id)
        }
    }
}

async fn process_sat_file_image_product_type_ims_recipe(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    recipe_id: &str,
    image_name: &str,
) -> Result<String, Error> {
    /* let recipe_id: String = product_details
    .as_object()
    .unwrap()
    .values()
    .collect::<Vec<_>>()
    .first()
    .unwrap()["id"]
    .as_str()
    .unwrap()
    .to_string(); */

    // Get root public ssh key
    let root_public_ssh_key_value: serde_json::Value = ims::image::utils::get_single(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        "mgmt root key",
    )
    .await
    .unwrap();

    let root_public_ssh_key = root_public_ssh_key_value["id"].as_str().unwrap();

    let ims_job = ims::job::r#struct::JobPostRequest {
        job_type: "create".to_string(),
        image_root_archive_name: image_name.to_string(),
        kernel_file_name: Some("vmlinuz".to_string()),
        initrd_file_name: Some("initrd".to_string()),
        kernel_parameters_file_name: Some("kernel-parameters".to_string()),
        artifact_id: recipe_id.to_string(),
        public_key_id: root_public_ssh_key.to_string(),
        ssh_containers: None, // Should this be None ???
        enable_debug: Some(false),
        build_env_size: Some(15),
    };

    let ims_job: serde_json::Value =
        ims::job::http_client::post_sync(shasta_token, shasta_base_url, shasta_root_cert, &ims_job)
            .await
            .unwrap();

    Ok(ims_job["resultant_image_id"].as_str().unwrap().to_string())
}

async fn process_sat_file_image_ims_type_recipe(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    sat_file_image_base_ims_value_yaml: &serde_yaml::Value,
    image_name: &String,
) -> Result<String, Error> {
    // Base image needs to be created from a IMS job using an IMS recipe
    let recipe_name = sat_file_image_base_ims_value_yaml["name"].as_str().unwrap();

    // Get all IMS recipes
    let recipe_detail_vec: Vec<ims::recipe::r#struct::RecipeGetResponse> =
        ims::recipe::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, None)
            .await
            .unwrap();

    // Filter recipes by name
    let recipe_detail_opt = recipe_detail_vec
        .iter()
        .find(|recipe| recipe.name == recipe_name);

    log::info!("IMS recipe details:\n{:#?}", recipe_detail_opt);

    // Check recipe with requested name exists
    let recipe_id = if let Some(recipe_detail) = recipe_detail_opt {
        recipe_detail.id.as_ref().unwrap()
    } else {
        return Err(Error::Message(format!(
            "IMS recipe with name '{}' - not found. Exit",
            recipe_name
        )));
    };

    log::info!("IMS recipe id found '{}'", recipe_id);

    // Get root public ssh key
    let root_public_ssh_key_value: serde_json::Value = ims::image::utils::get_single(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        "mgmt root key",
    )
    .await
    .unwrap();

    let root_public_ssh_key = root_public_ssh_key_value["id"].as_str().unwrap();

    let ims_job = ims::job::r#struct::JobPostRequest {
        job_type: "create".to_string(),
        image_root_archive_name: image_name.to_string(),
        kernel_file_name: Some("vmlinuz".to_string()),
        initrd_file_name: Some("initrd".to_string()),
        kernel_parameters_file_name: Some("kernel-parameters".to_string()),
        artifact_id: recipe_id.to_string(),
        public_key_id: root_public_ssh_key.to_string(),
        ssh_containers: None, // Should this be None ???
        enable_debug: Some(false),
        build_env_size: Some(15),
    };

    let ims_job: serde_json::Value =
        ims::job::http_client::post_sync(shasta_token, shasta_base_url, shasta_root_cert, &ims_job)
            .await
            .unwrap();

    log::info!("IMS job response:\n{:#?}", ims_job);

    Ok(ims_job["resultant_image_id"].as_str().unwrap().to_string())
}

fn process_sat_file_image_old_version(
    sat_file_image_ims_value_yaml: &serde_yaml::Value,
) -> Result<String, Error> {
    if sat_file_image_ims_value_yaml
        .get("is_recipe")
        .is_some_and(|is_recipe_value| is_recipe_value.as_bool().unwrap() == false)
        && sat_file_image_ims_value_yaml.get("id").is_some()
    {
        // Create final image from CFS session
        Ok(sat_file_image_ims_value_yaml["id"]
            .as_str()
            .unwrap()
            .to_string())
    } else {
        Err(Error::Message("Functionality not built. Exit".to_string()))
    }
}

fn process_sat_file_image_ref_name(
    sat_file_image_base_image_ref_value_yaml: &serde_yaml::Value,
    ref_name_image_id_hashmap: &HashMap<String, String>,
) -> Result<String, Error> {
    let image_ref: String = sat_file_image_base_image_ref_value_yaml
        .as_str()
        .unwrap()
        .to_string();

    // Process image with 'image_ref' from another image in this same SAT file
    Ok(ref_name_image_id_hashmap
        .get(&image_ref)
        .unwrap()
        .to_string())
}

pub fn filter_product_catalog_images(
    filter: &Value,
    image_map: Map<String, serde_json::Value>,
    image_name: &str,
) -> Result<String, Error> {
    if let Some(arch) = filter.get("arch") {
        // Search image in product catalog and filter by arch
        let image_key_vec = image_map
            .keys()
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|product| {
                product
                    .split(".")
                    .last()
                    .unwrap()
                    .eq(arch.as_str().unwrap())
            })
            .collect::<Vec<_>>();

        if image_key_vec.is_empty() {
            Err(Error::Message(format!(
                "Product catalog for image '{}' not found. Exit",
                image_name
            )))
        } else if image_key_vec.len() > 1 {
            Err(Error::Message(format!(
                "Product catalog for image '{}' multiple items found. Exit",
                image_name
            )))
        } else {
            let image_key = image_key_vec.first().cloned().unwrap();
            Ok(image_map.get(image_key).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string())
        }
    } else if let Some(wildcard) = filter.get("wildcard") {
        // Search image in product catalog and filter by wildcard
        let image_key_vec = image_map
            .keys()
            .filter(|product| product.contains(wildcard.as_str().unwrap()))
            .collect::<Vec<_>>();

        if image_key_vec.is_empty() {
            Err(Error::Message(format!(
                "Product catalog for image '{}' not found. Exit",
                image_name
            )))
        } else if image_key_vec.len() > 1 {
            Err(Error::Message(format!(
                "Product catalog for image '{}' multiple items found. Exit",
                image_name
            )))
        } else {
            let image_key = image_key_vec.first().cloned().unwrap();
            Ok(image_map.get(image_key).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string())
        }
    } else if let Some(prefix) = filter.get("prefix") {
        // Search image in product catalog and filter by prefix
        let image_key_vec = image_map
            .keys()
            .filter(|product| {
                product
                    .strip_prefix(&prefix.as_str().unwrap().to_string())
                    .is_some()
            })
            .collect::<Vec<_>>();

        if image_key_vec.is_empty() {
            Err(Error::Message(format!(
                "Product catalog for image '{}' not found. Exit",
                image_name
            )))
        } else if image_key_vec.len() > 1 {
            Err(Error::Message(format!(
                "Product catalog for image '{}' multiple items found. Exit",
                image_name
            )))
        } else {
            let image_key = image_key_vec.first().cloned().unwrap();
            Ok(image_map.get(image_key).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string())
        }
    } else {
        Err(Error::Message(format!(
            "Product catalog for image '{}' not found. Exit",
            image_name
        )))
    }
}

pub fn validate_sat_file_images_section(
    image_yaml_vec: &Vec<Value>,
    configuration_yaml_vec: &Vec<Value>,
    hsm_group_available_vec: &[String],
    cray_product_catalog: &BTreeMap<String, String>,
    image_vec: Vec<ims::image::r#struct::Image>,
    configuration_vec: Vec<CfsConfigurationResponse>,
    ims_recipe_vec: Vec<ims::recipe::r#struct::RecipeGetResponse>,
) -> Result<(), Error> {
    // Validate 'images' section in SAT file

    for image_yaml in image_yaml_vec {
        // Validate image
        let image_name = image_yaml["name"].as_str().unwrap();

        log::info!("Validate 'image' '{}'", image_name);

        // Validate base image
        log::info!("Validate 'image' '{}' base image", image_name);

        if let Some(image_ims_id_to_find) = image_yaml
            .get("ims")
            .and_then(|ims| ims.get("id").and_then(|id| id.as_str()))
        {
            // Old format
            log::info!(
                "Searching image.ims.id (old format - backward compatibility) '{}' in CSM",
                image_ims_id_to_find,
            );

            let is_image_base_id_in_csm =
                image_vec.iter().any(|image: &ims::image::r#struct::Image| {
                    let image_id = image.id.as_ref().unwrap();
                    image_id.eq(image_ims_id_to_find)
                });

            if !is_image_base_id_in_csm {
                return Err(Error::Message(format!(
                    "Could not find base image id '{}' in image '{}'. Exit",
                    image_ims_id_to_find,
                    image_yaml["name"].as_str().unwrap()
                )));
            }
        } else if image_yaml.get("base").is_some() {
            // New format
            if let Some(image_ref_to_find) = image_yaml["base"].get("image_ref") {
                // Check there is another image with 'ref_name' that matches this 'image_ref'
                let image_found = image_yaml_vec.iter().any(|image_yaml| {
                    image_yaml
                        .get("ref_name")
                        .is_some_and(|ref_name| ref_name.eq(image_ref_to_find))
                });

                if !image_found {
                    return Err(Error::Message(format!(
                                "Could not find image with ref name '{}' in SAT file. Cancelling image build proccess. Exit",
                                image_ref_to_find.as_str().unwrap(),
                            )));
                }
            } else if let Some(image_base_product) = image_yaml["base"].get("product") {
                // Check if the 'Cray/HPE product' in CSM exists

                log::info!("Image '{}' base.base.product", image_name);
                log::info!("SAT file - 'image.base.product' job");

                // Base image created from a cray product

                let product_name = image_base_product["name"].as_str().unwrap();

                let product_version = image_base_product["version"].as_str().unwrap();

                let product_type = image_base_product["type"].as_str().unwrap().to_string() + "s";

                let product_catalog_rslt = &serde_yaml::from_str::<serde_json::Value>(
                    &cray_product_catalog
                        .get(product_name)
                        .unwrap_or(&"".to_string()),
                );

                let product_catalog = if let Ok(product_catalog) = product_catalog_rslt {
                    product_catalog
                } else {
                    return Err(Error::Message(format!(
                        "Product catalog for image '{}' not found. Exit",
                        image_name
                    )));
                };

                let product_type_opt = product_catalog
                    .get(product_version)
                    .and_then(|product_version| product_version.get(product_type.clone()))
                    .cloned();

                let product_type_opt = if let Some(product_type) = product_type_opt {
                    product_type.as_object().cloned()
                } else {
                    return Err(Error::Message(format!(
                        "Product catalog for image '{}' not found. Exit",
                        image_name
                    )));
                };

                let image_map: Map<String, serde_json::Value> =
                    if let Some(product_type) = &product_type_opt {
                        product_type.clone()
                    } else {
                        return Err(Error::Message(format!(
                            "Product catalog for image '{}' not found. Exit",
                            image_name
                        )));
                    };

                log::debug!("CRAY product catalog items related to product name '{}', product version '{}' and product type '{}':\n{:#?}", product_name, product_version, product_type, product_type_opt);

                if let Some(filter) = image_base_product.get("filter") {
                    let image_recipe_id =
                        filter_product_catalog_images(filter, image_map, image_name);
                    image_recipe_id.is_ok()
                } else {
                    // There is no 'image.product.filter' value defined in SAT file. Check Cray
                    // product catalog only has 1 image. Othewise fail
                    log::info!("No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image");
                    image_map
                        .values()
                        .next()
                        .is_some_and(|value| value.get("id").is_some())
                };
            } else if let Some(image_base_ims_yaml) = image_yaml["base"].get("ims") {
                // Check if the image exists

                log::info!("Image '{}' base.base.ims", image_name);
                if let Some(image_base_ims_name_yaml) = image_base_ims_yaml.get("name") {
                    let image_base_ims_name_to_find = image_base_ims_name_yaml.as_str().unwrap();

                    // Search image in SAT file

                    log::info!(
                        "Searching base image '{}' related to image '{}' in SAT file",
                        image_base_ims_name_to_find,
                        image_name
                    );

                    let mut image_found = image_yaml_vec
                        .iter()
                        .any(|image_yaml| image_yaml["name"].eq(image_base_ims_name_yaml));

                    if !image_found {
                        log::warn!(
                            "Base image '{}' not found in SAT file, looking in CSM",
                            image_base_ims_name_to_find
                        );

                        if let Some(image_base_ims_type_yaml) = image_base_ims_yaml.get("type") {
                            let image_base_ims_type = image_base_ims_type_yaml.as_str().unwrap();
                            if image_base_ims_type.eq("recipe") {
                                // Base IMS type is a recipe
                                // Search in CSM (IMS Recipe)

                                log::info!(
                                    "Searching base image recipe '{}' related to image '{}' in CSM",
                                    image_base_ims_name_to_find,
                                    image_name
                                );

                                image_found = ims_recipe_vec
                                    .iter()
                                    .any(|recipe| recipe.name.eq(image_base_ims_name_to_find));

                                if !image_found {
                                    return Err(Error::Message(format!(
                                        "Could not find IMS recipe '{}' in CSM. Cancelling image build proccess. Exit",
                                        image_base_ims_name_to_find,
                                    )));
                                }
                            } else {
                                // Base IMS type is an image
                                // Search in CSM (IMS Image)

                                log::info!(
                                    "Searching base image '{}' related to image '{}' in CSM",
                                    image_base_ims_name_to_find,
                                    image_name
                                );

                                // CFS session sets a custom image name, therefore we can't seach
                                // for exact image name but search by substring
                                image_found = image_vec
                                    .iter()
                                    .any(|image| image.name.contains(image_base_ims_name_to_find));

                                if !image_found {
                                    return Err(Error::Message(format!(
                                        "Could not find image base '{}' in image '{}'. Cancelling image build proccess. Exit",
                                        image_base_ims_name_to_find,
                                        image_name
                                    )));
                                }
                            }
                        } else {
                            return Err(Error::Message(format!(
                                "Image '{}' is missing the field base.ims.type. Cancelling image build proccess. Exit",
                                image_base_ims_name_to_find,
                            )));
                        }
                    }
                } else {
                    eprintln!(
                        "Image '{}' is missing the field 'base.ims.name'. Exit",
                        image_name
                    );
                };
            } else {
                return Err(Error::Message(format!(
                    "Image '{}' yaml not recognised. Exit",
                    image_name
                )));
            }
        } else {
            return Err(Error::Message(format!(
                "Image '{}' neither have 'ims' nor 'base' value. Exit",
                image_name
            )));
        }

        // Validate CFS configuration exists (image.configuration)
        log::info!("Validate 'image' '{}' configuration", image_name);

        if let Some(configuration_yaml) = image_yaml.get("configuration") {
            let configuration_name_to_find = configuration_yaml.as_str().unwrap();

            log::info!(
                "Searching configuration name '{}' related to image '{}' in SAT file",
                configuration_name_to_find,
                image_name
            );

            let mut configuration_found = configuration_yaml_vec.iter().any(|configuration_yaml| {
                configuration_yaml["name"]
                    .as_str()
                    .unwrap()
                    .eq(configuration_name_to_find)
            });

            if !configuration_found {
                // CFS configuration in image not found in SAT file, searching in CSM
                log::warn!(
                    "Configuration '{}' not found in SAT file, looking in CSM",
                    configuration_name_to_find
                );

                log::info!(
                    "Searching configuration name '{}' related to image '{}' in CSM",
                    configuration_name_to_find,
                    image_yaml["name"].as_str().unwrap()
                );

                configuration_found = configuration_vec
                    .iter()
                    .any(|configuration| configuration.name.eq(configuration_name_to_find));

                if !configuration_found {
                    return Err(Error::Message(format!(
                        "Could not find configuration '{}' in image '{}'. Cancelling image build proccess. Exit",
                        configuration_name_to_find,
                        image_name
                    )));
                }
            }

            // Validate user has access to HSM groups in 'image' section
            log::info!("Validate 'image' '{}' HSM groups", image_name);

            let configuration_group_names_vec: Vec<String> =
                serde_yaml::from_value(image_yaml["configuration_group_names"].clone())
                    .unwrap_or(Vec::new());

            //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
            //wide operations instead of using roles
            let configuration_group_names_vec =
                csm_rs::hsm::group::hacks::filter_system_hsm_group_names(
                    configuration_group_names_vec,
                );

            if configuration_group_names_vec.is_empty() {
                return Err(Error::Message(format!("Image '{}' must have group name values assigned to it. Canceling image build process. Exit", image_name)));
            } else {
                for hsm_group in configuration_group_names_vec.iter().filter(|&hsm_group| {
                    !hsm_group.eq_ignore_ascii_case("Compute")
                        && !hsm_group.eq_ignore_ascii_case("Application")
                        && !hsm_group.eq_ignore_ascii_case("Application_UAN")
                }) {
                    if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                        return Err(Error::Message(format!
                        (
                        "HSM group '{}' in image '{}' not allowed, List of HSM groups available:\n{:?}. Exit",
                        hsm_group,
                        image_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    )));
                    }
                }
            };
        }
    }

    Ok(())
}

pub fn validate_sat_file_configurations_section(
    configuration_yaml_vec_opt: Option<&Vec<Value>>,
    image_yaml_vec_opt: Option<&Vec<Value>>,
    sessiontemplate_yaml_vec_opt: Option<&Vec<Value>>,
) {
    // Validate 'configurations' sections
    if configuration_yaml_vec_opt.is_some() && !configuration_yaml_vec_opt.unwrap().is_empty() {
        if !(image_yaml_vec_opt.is_some() && !image_yaml_vec_opt.unwrap().is_empty())
            && !(sessiontemplate_yaml_vec_opt.is_some()
                && !sessiontemplate_yaml_vec_opt.unwrap().is_empty())
        {
            eprint!(
                "Incorrect SAT file. Please define either an 'image' or a 'session template'. Exit"
            );
            std::process::exit(1);
        }
    }
}

pub async fn validate_sat_file_session_template_section(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_yaml_vec_opt: Option<&Vec<Value>>,
    configuration_yaml_vec_opt: Option<&Vec<Value>>,
    session_template_yaml_vec_opt: Option<&Vec<Value>>,
    hsm_group_available_vec: &Vec<String>,
) {
    // Validate 'session_template' section in SAT file
    log::info!("Validate 'session_template' section in SAT file");
    for session_template_yaml in session_template_yaml_vec_opt.unwrap_or(&vec![]) {
        // Validate session_template
        let session_template_name = session_template_yaml["name"].as_str().unwrap();

        log::info!("Validate 'session_template' '{}'", session_template_name);

        // Validate user has access to HSM groups in 'session_template' section
        log::info!(
            "Validate 'session_template' '{}' HSM groups",
            session_template_name
        );

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        for hsm_group in bos_session_template_hsm_groups {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                eprintln!(
                        "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(1);
            }
        }

        // Validate boot image (session_template.image)
        log::info!(
            "Validate 'session_template' '{}' boot image",
            session_template_name
        );

        if let Some(ref_name_to_find) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("image_ref"))
        {
            // Validate image_ref (session_template.image.image_ref). Search in SAT file for any
            // image with images[].ref_name
            log::info!(
                "Searching ref_name '{}' in SAT file",
                ref_name_to_find.as_str().unwrap(),
            );

            let image_ref_name_found = image_yaml_vec_opt.is_some_and(|image_vec| {
                image_vec.iter().any(|image| {
                    image
                        .get("ref_name")
                        .is_some_and(|ref_name| ref_name.eq(ref_name_to_find))
                })
            });

            if !image_ref_name_found {
                eprintln!(
                    "Could not find image ref '{}' in SAT file. Exit",
                    ref_name_to_find.as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_name_substr_to_find) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("ims").and_then(|ims| ims.get("name")))
        {
            // Validate image name (session_template.image.ims.name). Search in SAT file and CSM
            log::info!(
                "Searching image name '{}' related to session template '{}' in SAT file",
                image_name_substr_to_find.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let mut image_found = image_yaml_vec_opt.is_some_and(|image_vec| {
                image_vec.iter().any(|image| {
                    image
                        .get("name")
                        .is_some_and(|name| name.eq(image_name_substr_to_find))
                })
            });

            if !image_found {
                // image not found in SAT file, looking in CSM
                log::warn!(
                    "Image name '{}' not found in SAT file, looking in CSM",
                    image_name_substr_to_find.as_str().unwrap()
                );
                log::info!(
                    "Searching image name '{}' related to session template '{}' in CSM",
                    image_name_substr_to_find.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );

                image_found = csm_rs::ims::image::utils::get_fuzzy(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_available_vec,
                    image_name_substr_to_find.as_str(),
                    Some(&1),
                )
                .await
                .is_ok();
            }

            if !image_found {
                eprintln!(
                    "Could not find image name '{}' in session_template '{}'. Exit",
                    image_name_substr_to_find.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_id) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("ims").and_then(|ims| ims.get("id")))
        {
            // Validate image id (session_template.image.ims.id). Search in SAT file and CSM
            log::info!(
                "Searching image id '{}' related to session template '{}' in CSM",
                image_id.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let image_found = csm_rs::ims::image::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                image_id.as_str(),
            )
            .await
            .is_ok();

            if !image_found {
                eprintln!(
                    "Could not find image id '{}' in session_template '{}'. Exit",
                    image_id.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_name_substr_to_find) = session_template_yaml.get("image") {
            // Backward compatibility
            // VaVjlidate image name (session_template.image.ims.name). Search in SAT file and CSM
            log::info!(
                "Searching image name '{}' related to session template '{}' in CSM - ('sessiontemplate' section in SAT file is outdated - switching to backward compatibility)",
                image_name_substr_to_find.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let image_found = csm_rs::ims::image::utils::get_fuzzy(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_available_vec,
                image_name_substr_to_find.as_str(),
                Some(&1),
            )
            .await
            .is_ok();

            if !image_found {
                // image not found in SAT file, looking in CSM
                log::warn!(
                    "Image name '{}' not found in CSM. Exit",
                    image_name_substr_to_find.as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else {
            eprintln!(
                "Session template '{}' must have one of these entries 'image.ref_name', 'image.ims.name' or 'image.ims.id' values. Exit",
                session_template_yaml["name"].as_str().unwrap(),
            );
            std::process::exit(1);
        }

        // Validate configuration
        log::info!(
            "Validate 'session_template' '{}' configuration",
            session_template_name
        );

        if let Some(configuration_to_find_value) = session_template_yaml.get("configuration") {
            let configuration_to_find = configuration_to_find_value.as_str().unwrap();

            log::info!(
                "Searching configuration name '{}' related to session template '{}' in CSM in SAT file",
                configuration_to_find,
                session_template_yaml["name"].as_str().unwrap()
            );

            let mut configuration_found =
                configuration_yaml_vec_opt.is_some_and(|configuration_yaml_vec| {
                    configuration_yaml_vec.iter().any(|configuration_yaml| {
                        configuration_yaml["name"].eq(configuration_to_find_value)
                    })
                });

            if !configuration_found {
                // CFS configuration in session_template not found in SAT file, searching in CSM
                log::warn!("Configuration not found in SAT file, looking in CSM");
                log::info!(
                    "Searching configuration name '{}' related to session_template '{}' in CSM",
                    configuration_to_find,
                    session_template_yaml["name"].as_str().unwrap()
                );

                configuration_found = cfs::configuration::shasta::http_client::v2::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(configuration_to_find),
                )
                .await
                .is_ok();

                if !configuration_found {
                    eprintln!(
                        "ERROR - Could not find configuration '{}' in session_template '{}'. Exit",
                        configuration_to_find,
                        session_template_yaml["name"].as_str().unwrap(),
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!(
                "Session template '{}' does not have 'configuration' value. Exit",
                session_template_yaml["name"].as_str().unwrap(),
            );
            std::process::exit(1);
        }
    }
}

pub async fn process_session_template_section_in_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ref_name_processed_hashmap: HashMap<String, String>,
    _hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    sat_file_yaml: Value,
    do_not_reboot: bool,
    dry_run: bool,
    kafka_audit: &Kafka,
) {
    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    let mut bos_st_created_vec: Vec<String> = Vec::new();

    for bos_sessiontemplate_yaml in bos_session_template_list_yaml {
        let _bos_sessiontemplate: BosSessionTemplate =
            serde_yaml::from_value(bos_sessiontemplate_yaml.clone()).unwrap();

        let image_details: ims::image::r#struct::Image = if let Some(bos_sessiontemplate_image) =
            bos_sessiontemplate_yaml.get("image")
        {
            if let Some(bos_sessiontemplate_image_ims) = bos_sessiontemplate_image.get("ims") {
                // Get boot image to configure the nodes
                if let Some(bos_session_template_image_ims_name) =
                    bos_sessiontemplate_image_ims.get("name")
                {
                    // BOS sessiontemplate boot image defined by name
                    let bos_session_template_image_name = bos_session_template_image_ims_name
                        .as_str()
                        .unwrap()
                        .to_string();

                    // Get base image details
                    ims::image::utils::get_fuzzy(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_available_vec,
                        Some(&bos_session_template_image_name),
                        Some(&1),
                    )
                    .await
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                } else if let Some(bos_session_template_image_ims_id) =
                    bos_sessiontemplate_image_ims.get("id")
                {
                    // BOS sessiontemplate boot image defined by id
                    let bos_session_template_image_id = bos_session_template_image_ims_id
                        .as_str()
                        .unwrap()
                        .to_string();

                    // Get base image details
                    ims::image::csm_rs::http_client::get(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(&bos_session_template_image_id),
                    )
                    .await
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                } else {
                    eprintln!("ERROR: neither 'image.ims.name' nor 'image.ims.id' fields defined in session_template.\nExit");
                    std::process::exit(1);
                }
            } else if let Some(bos_session_template_image_image_ref) =
                bos_sessiontemplate_image.get("image_ref")
            {
                // BOS sessiontemplate boot image defined by image_ref
                let image_ref = bos_session_template_image_image_ref
                    .as_str()
                    .unwrap()
                    .to_string();

                let image_id = ref_name_processed_hashmap
                    .get(&image_ref)
                    .unwrap()
                    .to_string();

                // Get Image by id
                ims::image::csm_rs::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&image_id),
                )
                .await
                .unwrap()
                .first()
                .unwrap()
                .clone()
            } else if let Some(image_name_substring) = bos_sessiontemplate_image.as_str() {
                let image_name = image_name_substring;
                // Backward compatibility
                // Get base image details
                log::info!("Looking for IMS image which name contains '{}'", image_name);

                if !dry_run {
                    let image_vec = ims::image::utils::get_fuzzy(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_available_vec,
                        Some(&image_name),
                        None,
                    )
                    .await
                    .unwrap();

                    // Validate/check if image exists
                    if image_vec.is_empty() {
                        eprintln!(
                            "ERROR: Could not find an image which name contains '{}'. Exit",
                            image_name
                        );
                        std::process::exit(1);
                    };

                    image_vec.first().unwrap().clone()
                } else {
                    csm_rs::ims::image::r#struct::Image {
                        id: None,
                        created: None,
                        name: image_name.to_string(),
                        link: None,
                        arch: None,
                    }
                }
            } else {
                eprintln!("ERROR: neither 'image.ims' nor 'image.image_ref' sections found in session_template.image.\nExit");
                std::process::exit(1);
            }
        } else {
            eprintln!("ERROR: no 'image' section in session_template.\nExit");
            std::process::exit(1);
        };

        log::info!("Image with name '{}' found", image_details.name);

        // Get CFS configuration to configure the nodes
        let bos_session_template_configuration_name = bos_sessiontemplate_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string();

        // bos_session_template_configuration_name.replace("__DATE__", tag);

        log::info!(
            "Looking for CFS configuration with name: {}",
            bos_session_template_configuration_name
        );

        if !dry_run {
            let cfs_configuration_vec_rslt = cfs::configuration::csm_rs::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(&bos_session_template_configuration_name),
            )
            .await;

            /* csm_rs::cfs::configuration::csm_rs::utils::filter(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_configuration_vec,
                None,
                hsm_group_available_vec,
                None,
            )
            .await; */

            if cfs_configuration_vec_rslt.is_err() || cfs_configuration_vec_rslt.unwrap().is_empty()
            {
                eprintln!(
                    "ERROR: BOS session template configuration not found in SAT file image list."
                );
                std::process::exit(1);
            }
        } else {
        }

        let _ims_image_name = image_details.name.to_string();
        let ims_image_etag = image_details.link.as_ref().unwrap().etag.as_ref().unwrap();
        let ims_image_path = &image_details.link.as_ref().unwrap().path;
        let ims_image_type = &image_details.link.as_ref().unwrap().r#type;

        let bos_sessiontemplate_name = bos_sessiontemplate_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // bos_session_template_name.replace("__DATE__", tag);

        let mut boot_set_vec: HashMap<String, BootSet> = HashMap::new();

        for (parameter, boot_set) in bos_sessiontemplate_yaml["bos_parameters"]["boot_sets"]
            .as_mapping()
            .unwrap()
        {
            let kernel_parameters = boot_set["kernel_parameters"].as_str().unwrap();
            let arch_opt = boot_set["arch"].as_str().map(|value| value.to_string());

            let node_roles_groups_opt: Option<Vec<String>> = boot_set
                .get("node_roles_groups")
                .and_then(|node_roles_groups| {
                    node_roles_groups
                        .as_sequence()
                        .and_then(|node_role_groups| {
                            node_role_groups
                                .iter()
                                .map(|hsm_group_value| {
                                    hsm_group_value
                                        .as_str()
                                        .map(|hsm_group| hsm_group.to_string())
                                })
                                .collect()
                        })
                });

            // Validate/check user can create BOS sessiontemplates based on node roles. Users
            // with tenant role are not allowed to create BOS sessiontemplates based on node roles
            // however admin tenants are allowed to create BOS sessiontemplates based on node roles
            if !hsm_group_available_vec.is_empty()
                && node_roles_groups_opt
                    .clone()
                    .is_some_and(|node_roles_groups| !node_roles_groups.is_empty())
            {
                eprintln!("User type tenant can't user node roles in BOS sessiontemplate. Exit");
                std::process::exit(1);
            }

            let node_groups_opt: Option<Vec<String>> =
                boot_set.get("node_groups").and_then(|node_groups_value| {
                    node_groups_value.as_sequence().and_then(|node_group| {
                        node_group
                            .iter()
                            .map(|hsm_group_value| {
                                hsm_group_value
                                    .as_str()
                                    .map(|hsm_group| hsm_group.to_string())
                            })
                            .collect()
                    })
                });

            //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
            //wide operations instead of using roles
            let node_groups_opt = Some(csm_rs::hsm::group::hacks::filter_system_hsm_group_names(
                node_groups_opt.unwrap_or_default(),
            ));

            // Validate/check HSM groups in YAML file session_templates.bos_parameters.boot_sets.<parameter>.node_groups matches with
            // Check hsm groups in SAT file includes the hsm_group_param
            for node_group in node_groups_opt.clone().unwrap_or_default() {
                if !hsm_group_available_vec.contains(&node_group.to_string()) {
                    eprintln!("User does not have access to HSM group '{}' in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Exit", node_group);
                    std::process::exit(1);
                }
            }

            // Validate user has access to the xnames in the BOS sessiontemplate
            let node_list_opt: Option<Vec<String>> =
                boot_set.get("node_list").and_then(|node_list_value| {
                    node_list_value.as_sequence().and_then(|node_list| {
                        node_list
                            .into_iter()
                            .map(|node_value_value| {
                                node_value_value
                                    .as_str()
                                    .map(|node_value| node_value.to_string())
                            })
                            .collect()
                    })
                });

            // Validate user has access to the list of nodes in BOS sessiontemplate
            if let Some(node_list) = &node_list_opt {
                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    node_list.iter().map(|node| node.to_string()).collect(),
                )
                .await;
            }

            let cfs = Cfs {
                // clone_url: None,
                // branch: None,
                // commit: None,
                // playbook: None,
                configuration: Some(bos_session_template_configuration_name.clone()),
            };

            // let rootfs_provider = Some("cpss3".to_string());
            let rootfs_provider = boot_set["rootfs_provider"]
                .as_str()
                .map(|value| value.to_string());
            let rootfs_provider_passthrough = boot_set["rootfs_provider_passthrough"]
                .as_str()
                .map(|value| value.to_string());

            let boot_set = BootSet {
                name: None,
                // boot_ordinal: Some(2),
                // shutdown_ordinal: None,
                path: Some(ims_image_path.to_string()),
                r#type: Some(ims_image_type.to_string()),
                etag: Some(ims_image_etag.to_string()),
                kernel_parameters: Some(kernel_parameters.to_string()),
                // network: Some("nmn".to_string()),
                node_list: node_list_opt,
                node_roles_groups: node_roles_groups_opt, // TODO: investigate whether this value can be a list
                // of nodes and if it is process it properly
                node_groups: node_groups_opt,
                rootfs_provider,
                rootfs_provider_passthrough,
                cfs: Some(cfs),
                arch: arch_opt,
            };

            boot_set_vec.insert(parameter.as_str().unwrap().to_string(), boot_set);
        }

        /* let create_bos_session_template_payload = BosSessionTemplate::new_for_hsm_group(
            bos_session_template_configuration_name,
            bos_session_template_name,
            ims_image_name,
            ims_image_path.to_string(),
            ims_image_type.to_string(),
            ims_image_etag.to_string(),
            hsm_group,
        ); */

        let cfs = Cfs {
            // clone_url: None,
            // branch: None,
            // commit: None,
            // playbook: None,
            configuration: Some(bos_session_template_configuration_name),
        };

        let create_bos_session_template_payload = BosSessionTemplate {
            // template_url: None,
            // name: Some(bos_sessiontemplate_name.clone()),
            name: None,
            description: None,
            // cfs_url: None,
            // cfs_branch: None,
            enable_cfs: Some(true),
            cfs: Some(cfs),
            // partition: None,
            boot_sets: Some(boot_set_vec),
            links: None,
            tenant: None,
        };

        if !dry_run {
            let create_bos_session_template_resp = bos::template::shasta::http_client::v2::put(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &create_bos_session_template_payload,
                // &create_bos_session_template_payload.name.as_ref().unwrap(),
                &bos_sessiontemplate_name,
            )
            .await;

            match create_bos_session_template_resp {
                Ok(bos_sessiontemplate) => {
                    println!(
                        "BOS sessiontemplate name '{}' created",
                        bos_sessiontemplate_name
                    );

                    bos_st_created_vec.push(bos_sessiontemplate.name.unwrap())
                }
                Err(error) => eprintln!(
                    "ERROR: BOS session template creation failed.\nReason:\n{}\nExit",
                    error
                ),
            }
        } else {
            println!(
                "BOS sessiontemplate to create:\n{:#?}",
                create_bos_session_template_payload
            );
        }
    }

    // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
    // up... hence we will split the reboot into 2 operations shutdown and start

    if do_not_reboot {
        log::info!("Reboot canceled by user");
    } else {
        log::info!("Rebooting");

        for bos_st_name in bos_st_created_vec {
            log::info!(
                "Creating BOS session for BOS sessiontemplate '{}' to reboot",
                bos_st_name
            );

            // BOS session v1
            /* let create_bos_session_resp = bos::session::shasta::http_client::v1::post(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &bos_st_name,
                "reboot",
                None,
            )
            .await; */

            // BOS session v2
            let bos_session = BosSession {
                name: None,
                tenant: None,
                operation: Some(Operation::Reboot),
                template_name: bos_st_name.clone(),
                limit: None,
                stage: None,
                include_disabled: None,
                status: None,
                components: None,
            };

            let create_bos_session_resp = bos::session::shasta::http_client::v2::post(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_session,
            )
            .await;

            match create_bos_session_resp {
                Ok(bos_session) => {
                    // log::info!("K8s job relates to BOS session v1 '{}'", bos_session["job"].as_str().unwrap());
                    println!(
                        "BOS session '{}' for BOS sessiontemplate '{}' created",
                        bos_session["name"].as_str().unwrap(),
                        bos_st_name
                    )
                }
                Err(error) => eprintln!(
                    "ERROR: BOS session for BOS sessiontemplate '{}' creation failed.\nReason:\n{}\nExit",
                    bos_st_name,
                    error
                ),
            }

            let bos_sessiontemplate_vec = bos::template::csm_rs::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(&bos_st_name),
            )
            .await
            .unwrap();

            let bos_sessiontemplate = bos_sessiontemplate_vec.first().unwrap();

            let _ = if !bos_sessiontemplate.get_target_hsm().is_empty() {
                // Get list of XNAMES for all HSM groups
                let mut xnames = Vec::new();
                for hsm in bos_sessiontemplate.get_target_hsm().iter() {
                    xnames.append(
                        &mut hsm::group::utils::get_member_vec_from_hsm_group_name(
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
                            hsm,
                        )
                        .await,
                    );
                }

                xnames
            } else {
                // Get list of XNAMES
                bos_sessiontemplate.get_target_xname()
            };

            // power_reset_nodes::exec(
            //     shasta_token,
            //     shasta_base_url,
            //     shasta_root_cert,
            //     xnames,
            //     Some("Force BOS session reboot".to_string()),
            //     true,
            // )
            // .await;
        }
    }

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "message": "Apply SAT file"});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
