use std::collections::{BTreeMap, HashMap};

use mesa::{
    bos::template::mesa::r#struct::v1::{BootSet, BosSessionTemplate, Cfs},
    cfs::{
        self,
        configuration::mesa::r#struct::{
            cfs_configuration_request::v2::CfsConfigurationRequest,
            cfs_configuration_response::v2::CfsConfigurationResponse,
        },
        session::mesa::r#struct::v2::CfsSessionPostRequest,
    },
    error::Error,
    ims::{self, image::r#struct::Image, recipe::r#struct::RecipeGetResponse},
};
use serde_json::Map;
use serde_yaml::{Mapping, Value};

use self::sat_file_bos_sessiontemplate::SessionTemplate;

/// struct to represent the `session_templates` section in SAT file
pub mod sat_file_bos_sessiontemplate {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct SessionTemplate {
        pub name: String,
        pub image: Image,
        pub configuration: String,
        pub bos_parameters: BosParamters,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub enum Image {
        Ims(ImsAux),
        ImageRef(String),
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub enum ImsAux {
        Name(String),
        Id(String),
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct BosParamters {
        pub boot_sets: HashMap<String, BootSet>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct BootSet {
        pub arch: Option<Arch>,
        pub kernel_parameters: Option<String>,
        pub network: Option<String>,
        pub node_list: Option<String>,
        pub node_roles_group: Option<String>,
        pub node_groups: Option<String>,
        pub rootfs_provider: Option<String>,
        pub rootfs_provider_passthrough: Option<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub enum Arch {
        X86,
        ARM,
        Other,
    }
}

/// Convert from `sessiontemplate` in SAT file to manta BosSessionTemplate
/// example from https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html
impl TryFrom<SessionTemplate> for BosSessionTemplate {
    type Error = ();

    fn try_from(value: SessionTemplate) -> Result<BosSessionTemplate, Self::Error> {
        let b_st_cfs = Cfs {
            clone_url: None,
            branch: None,
            commit: None,
            playbook: None,
            configuration: Some(value.configuration),
        };

        let mut boot_set_map: HashMap<String, BootSet> = HashMap::new();

        for (property, boot_set) in value.bos_parameters.boot_sets {
            let boot_set = BootSet {
                name: Some(format!(
                    "Boot set property '{}' created by manta from SAT file",
                    property
                )),
                boot_ordinal: Some(2),
                shutdown_ordinal: None,
                path: None,
                r#type: None,
                etag: None,
                kernel_parameters: None,
                network: Some("nmn".to_string()),
                node_list: boot_set.node_list.map(|node_list| {
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
                }),
                rootfs_provider: Some("cpss3".to_string()),
                rootfs_provider_passthrough: boot_set.rootfs_provider_passthrough,
            };

            boot_set_map.insert(property, boot_set);
        }

        let b_st = BosSessionTemplate {
            template_url: None,
            name: value.name,
            description: Some(format!(
                "BOS sessiontemplate created by manta from SAT file"
            )),
            cfs_url: None,
            cfs_branch: None,
            enable_cfs: Some(true),
            cfs: Some(b_st_cfs),
            partition: None,
            boot_sets: Some(boot_set_map),
            links: None,
        };

        Ok(b_st)
    }
}

/// struct to represent the `images` section in SAT file
pub mod sat_file_image {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Ims {
        name: String,
        r#type: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Product {
        name: String,
        version: String,
        r#type: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
    pub enum ImageBase {
        Ims { ims: Ims },
        Product { product: Product },
        ImageRef { image_ref: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct SatFileImage {
        pub name: String,
        pub base: ImageBase,
        pub configuration: Option<String>,
        pub configuration_group_names: Option<Vec<String>>,
        pub ref_name: Option<String>,
        pub description: Option<String>,
    }
}

/// struct to represent the `configurations` section in SAT file
pub mod sat_file_configuration {
    use serde::{Deserialize, Serialize};

    pub enum ProductType {
        ProductVersion {
            name: String,
            version: String,
        },
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
    }

    pub struct Product {
        name: Option<String>,
        playbook: Option<String>,
        special_parameter: Option<String>,
        product: ProductType,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub enum Git {
        GitCommit { url: String, commit: String },
        GitBranch { url: String, branch: String },
        GitTag { url: String, tag: String },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Layer {
        pub name: Option<String>,
        #[serde(default = "default_playbook")]
        pub playbook: String, // This field is optional but with default value. Therefore we won't
        // make it optional
        pub git: Git,
    }

    fn default_playbook() -> String {
        "site.yml".to_string()
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub enum Inventory {
        InventoryCommit {
            name: Option<String>,
            url: String,
            commit: String,
        },
        InventoryBranch {
            name: Option<String>,
            url: String,
            branch: String,
        },
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Configuration {
        pub name: String,
        pub description: Option<String>,
        pub layers: Vec<Layer>,
        pub additional_inventory: Inventory,
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
    let env = minijinja::Environment::new();

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

    if let Some(value_option_vec) = value_cli_vec_opt {
        for value_option in value_option_vec {
            let cli_var_context_yaml = dot_notation_to_yaml(&value_option).unwrap();
            values_file_yaml = merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).unwrap();
        }
    }

    // render sat template file
    let sat_file_rendered = env.render_str(sat_file_content, values_file_yaml).unwrap();

    let sat_file_yaml: Value = serde_yaml::from_str::<Value>(&sat_file_rendered).unwrap();

    sat_file_yaml
}

pub async fn create_cfs_configuration_from_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    gitea_token: &str,
    cray_product_catalog: &BTreeMap<String, String>,
    sat_file_configuration_yaml: &serde_yaml::Value,
    // tag: &str,
) -> Result<CfsConfigurationResponse, Error> {
    log::debug!(
        "Convert CFS configuration in SAT file (yaml):\n{:#?}",
        sat_file_configuration_yaml
    );

    let mut cfs_configuration = CfsConfigurationRequest::from_sat_file_serde_yaml(
        shasta_root_cert,
        gitea_token,
        sat_file_configuration_yaml,
        cray_product_catalog,
    )
    .await;

    cfs::configuration::mesa::utils::create(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_configuration,
    )
    .await
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
    ref_name_processed_hashmap: &mut HashMap<String, String>,
    image_yaml_vec: Vec<serde_yaml::Value>,
    cray_product_catalog: &BTreeMap<String, String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    // tag: &str,
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
            image_yaml,
            cray_product_catalog,
            ansible_verbosity_opt,
            ansible_passthrough_opt,
            ref_name_processed_hashmap,
            // tag,
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
    image_yaml: &serde_yaml::Value, // NOTE: image may be an IMS job or a CFS session
    cray_product_catalog: &BTreeMap<String, String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    ref_name_image_id_hashmap: &HashMap<String, String>,
    // tag: &str,
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
    let configuration: String = image_yaml["configuration"]
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

    //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
    //wide operations instead of using roles
    let groups_name = mesa::hsm::group::hacks::filter_system_hsm_group_names(groups_name);

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

    if configuration.is_empty() {
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
            configuration,
            None,
            ansible_verbosity_opt,
            ansible_passthrough_opt.cloned(),
            true,
            Some(groups_name.to_vec()),
            Some(base_image_id),
        );

        let cfs_session = cfs::session::mesa::http_client::post_sync(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_session,
        )
        .await
        .unwrap();

        if !cfs_session.is_success() {
            eprintln!(
                "Error: CFS session '{}' failed. Exit",
                cfs_session.name.unwrap()
            );
            std::process::exit(1);
        }

        let image_id = cfs_session.get_first_result_id().unwrap();
        println!("Image '{}' imported image_id '{}'", image_name, image_id);

        Ok(image_id)
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
        Some("mgmt root key"),
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
    let recipe_detail_vec: Vec<RecipeGetResponse> =
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
        Some("mgmt root key"),
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
    image_vec: Vec<Image>,
    configuration_vec: Vec<CfsConfigurationResponse>,
    ims_recipe_vec: Vec<RecipeGetResponse>,
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

            let is_image_base_id_in_csm = image_vec.iter().any(|image| {
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
                mesa::hsm::group::hacks::filter_system_hsm_group_names(
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
