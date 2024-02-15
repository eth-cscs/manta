use std::collections::{BTreeMap, HashMap};

use mesa::{
    cfs::{
        self,
        configuration::mesa::r#struct::cfs_configuration_response::{
            ApiError, CfsConfigurationResponse,
        },
        session::mesa::r#struct::CfsSessionPostRequest,
    },
    ims::{self, recipe::r#struct::RecipeGetResponse},
};

pub async fn create_cfs_configuration_from_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    gitea_token: &str,
    cray_product_catalog: &BTreeMap<String, String>,
    sat_file_configuration_yaml: &serde_yaml::Value,
    tag: &str,
) -> Result<CfsConfigurationResponse, ApiError> {
    let mut cfs_configuration = mesa::cfs::configuration::mesa::r#struct::cfs_configuration_request::CfsConfigurationRequest::from_sat_file_serde_yaml(
        shasta_root_cert,
        gitea_token,
        sat_file_configuration_yaml,
        cray_product_catalog,
    )
    .await;

    // Rename configuration name
    cfs_configuration.name = cfs_configuration.name.replace("__DATE__", tag);

    mesa::cfs::configuration::mesa::utils::create(
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
pub fn get_next_image_to_process(
    image_yaml_vec: &Vec<serde_yaml::Value>,
    ref_name_processed_vec: &Vec<String>,
) -> Option<serde_yaml::Value> {
    image_yaml_vec
        .iter()
        .find(|image_yaml| {
            let ref_name: &str = &get_ref_name(image_yaml); // Again, because we assume images in
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
pub fn get_ref_name(image_yaml: &serde_yaml::Value) -> String {
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
    tag: &str,
) -> HashMap<String, serde_yaml::Value> {
    // Get an image to process (the image either has no dependency or it's image dependency has
    // already ben processed)
    let mut next_image_to_process_opt: Option<serde_yaml::Value> = get_next_image_to_process(
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
            &image_yaml,
            &cray_product_catalog,
            ansible_verbosity_opt,
            ansible_passthrough_opt,
            &ref_name_processed_hashmap,
            tag,
        )
        .await
        .unwrap();

        image_processed_hashmap.insert(image_id.clone(), image_yaml.clone());

        ref_name_processed_hashmap.insert(get_ref_name(&image_yaml), image_id.clone());

        next_image_to_process_opt = get_next_image_to_process(
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
    tag: &str,
) -> Result<String, ApiError> {
    log::info!("Importing image");
    // Collect CFS session details from SAT file
    // Get CFS session name from SAT file
    let image_name = image_yaml["name"]
        .as_str()
        .unwrap()
        .to_string()
        .replace("__DATE__", tag);

    log::info!("Importing image '{}'", image_name);

    // Get CFS configuration related to CFS session in SAT file
    let mut configuration: String = image_yaml["configuration"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    // Rename session's configuration name
    configuration = configuration.replace("__DATE__", tag);

    // Get HSM groups related to CFS session in SAT file
    let groups_name: Vec<String> = image_yaml["configuration_group_names"]
        .as_sequence()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|group_name| group_name.as_str().unwrap().to_string())
        .collect();

    let base_image_id: String;

    // Get/process base image
    if let Some(sat_file_image_ims_value_yaml) = image_yaml.get("ims") {
        // ----------- BASE IMAGE - BACKWARD COMPATIBILITY WITH PREVIOUS SAT FILE
        log::info!("SAT file - 'image.ims' job - previous version to create images in SAT file (for backward compatibility)");

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
                return Err(ApiError::MesaError(
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

            let product_details =
                &serde_yaml::from_str::<serde_json::Value>(&cray_product_catalog[product_name])
                    .unwrap()[product_version][product_type.clone()];

            log::debug!("Recipe details:\n{:#?}", product_details);

            // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE RECIPE
            if product_type == "recipes" {
                // Create base image from an IMS job (the 'id' field in
                // images[].base.product.id is the id of the IMS recipe used to
                // build the new base image)

                log::info!("SAT file - 'image.base.product' job based on IMS recipes");

                base_image_id = process_sat_file_image_product_type_ims_recipe(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &product_details,
                    &image_name,
                )
                .await
                .unwrap();

                // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE IMAGE
            } else if product_type == "images" {
                // Base image already created and its id is available in the Cray
                // product catalog

                log::info!("SAT file - 'image.base.product' job based on IMS images");

                log::info!("Getting base image id from Cray product catalog");
                base_image_id = product_details
                    .as_object()
                    .unwrap()
                    .values()
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()["id"]
                    .as_str()
                    .unwrap()
                    .to_string();
            } else {
                return Err(ApiError::MesaError(
                    "Can't process SAT file, field 'images.base.product.type' must be either 'images' or 'recipes'. Exit".to_string(),
                ));
            }
        } else {
            return Err(ApiError::MesaError(
                "Can't process SAT file 'images.base.product' is missing. Exit".to_string(),
            ));
        }
    } else {
        return Err(ApiError::MesaError(
            "Can't process SAT file 'images.base' is missing. Exit".to_string(),
        ));
    }

    if configuration.is_empty() {
        log::info!("No CFS session needs to be created since there is no CFS configuration assigned to this image");
        println!(
            "Image '{}' imported image_id '{}'",
            image_name, base_image_id
        );
        return Ok(base_image_id);
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

        let image_id = cfs_session.get_result_id().unwrap();
        println!("Image '{}' imported image_id '{}'", image_name, image_id);

        return Ok(image_id);
    }
}

async fn process_sat_file_image_product_type_ims_recipe(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    product_details: &serde_json::Value,
    image_name: &String,
) -> Result<String, ApiError> {
    let recipe_id: String = product_details
        .as_object()
        .unwrap()
        .values()
        .collect::<Vec<_>>()
        .first()
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get root public ssh key
    let root_public_ssh_key_value: serde_json::Value = ims::public_keys::http_client::get_single(
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
        image_root_archive_name: image_name.clone(),
        kernel_file_name: Some("vmlinuz".to_string()),
        initrd_file_name: Some("initrd".to_string()),
        kernel_parameters_file_name: Some("kernel-parameters".to_string()),
        artifact_id: recipe_id,
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
) -> Result<String, ApiError> {
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
        return Err(ApiError::MesaError(format!(
            "IMS recipe with name '{}' - not found. Exit",
            recipe_name
        )));
    };

    log::info!("IMS recipe id found '{}'", recipe_id);

    // Get root public ssh key
    let root_public_ssh_key_value: serde_json::Value = ims::public_keys::http_client::get_single(
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
) -> Result<String, ApiError> {
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
        return Err(ApiError::MesaError(
            "Functionality not built. Exit".to_string(),
        ));
    }
}

fn process_sat_file_image_ref_name(
    sat_file_image_base_image_ref_value_yaml: &serde_yaml::Value,
    ref_name_image_id_hashmap: &HashMap<String, String>,
) -> Result<String, ApiError> {
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

#[cfg(test)]
mod tests {
    use crate::common::sat_file::{get_next_image_to_process, get_ref_name};

    /// Test function "get_ref_name" so it falls back to "name" field if "ref_name" is missing
    #[test]
    fn test_get_ref_name() {
        let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(
            r#"images:
               - name: base_image
                 base:
                   product: 
                     name: cos
                     type: recipe
                     version: "2.4.139"
            "#,
        )
        .unwrap();

        println!(
            "image yaml vec:\n{}",
            serde_yaml::to_string(&image_yaml_vec).unwrap()
        );

        let ref_name_processed_vec: Vec<String> = Vec::new();
        let next_image_to_process: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        let image_ref = get_ref_name(&next_image_to_process.unwrap());

        assert!(image_ref == "base_image");
    }

    /// Test function "get_next_image_to_process" in an images section is SAT file with one image with ref_name
    #[test]
    fn test_get_next_image_to_process_1() {
        let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(r#"images: []"#).unwrap();

        println!(
            "image yaml vec:\n{}",
            serde_yaml::to_string(&image_yaml_vec).unwrap()
        );

        let ref_name_processed_vec: Vec<String> = Vec::new();

        let next_image_to_process: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        println!(
            "next image to process:\n{}",
            serde_yaml::to_string(&next_image_to_process).unwrap()
        );

        assert!(next_image_to_process.is_none());
    }

    #[test]
    fn test_get_next_image_to_process_2() {
        let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(
            r#"images:
               - name: base_image
                 ref_name: base_cos_image
                 base:
                   product: 
                     name: cos
                     type: recipe
                     version: "2.4.139"
            "#,
        )
        .unwrap();

        println!(
            "image yaml vec:\n{}",
            serde_yaml::to_string(&image_yaml_vec).unwrap()
        );

        let ref_name_processed_vec: Vec<String> = Vec::new();
        let next_image_to_process: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        println!(
            "next image to process:\n{}",
            serde_yaml::to_string(&next_image_to_process).unwrap()
        );

        assert!(next_image_to_process.unwrap()["name"].as_str().unwrap() == "base_image");
    }

    /// Test function "get_next_image_to_process" in an images section in SAT file with 2 images.
    /// The test should pass if the first image to process is the one with no dependencies and the
    /// second is the one which depends on the first one
    #[test]
    fn test_get_next_image_to_process_3() {
        let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(
            r#"images:
               - name: base_image
                 ref_name: base_cos_image
                 base:
                   product: 
                     name: cos
                     type: recipe
                     version: "2.4.139"
               - name: final_image
                 ref_name: compute_image
                 base:
                    image_ref: base_cos_image
            "#,
        )
        .unwrap();

        println!(
            "image yaml vec:\n{}",
            serde_yaml::to_string(&image_yaml_vec).unwrap()
        );

        let mut ref_name_processed_vec: Vec<String> = Vec::new();

        let next_image_to_process_1: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        ref_name_processed_vec.push("base_cos_image".to_string());

        let next_image_to_process_2: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        assert!(
            next_image_to_process_1.unwrap()["name"].as_str().unwrap() == "base_image"
                && next_image_to_process_2.unwrap()["name"].as_str().unwrap() == "final_image"
        );
    }

    #[test]
    fn test_get_next_image_to_process_4() {
        let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(
            r#"images:
               - name: base_image
                 ref_name: base_cos_image
                 base:
                   product: 
                     name: cos
                     type: recipe
                     version: "2.4.139"
               - name: final_image
                 ref_name: compute_image
                 base:
                    image_ref: base_cos_image
            "#,
        )
        .unwrap();

        println!(
            "image yaml vec:\n{}",
            serde_yaml::to_string(&image_yaml_vec).unwrap()
        );

        let mut ref_name_processed_vec: Vec<String> = Vec::new();

        let next_image_to_process_1: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        ref_name_processed_vec.push("base_cos_image".to_string());

        let next_image_to_process_2: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        ref_name_processed_vec.push("compute_image".to_string());

        let next_image_to_process_3: Option<serde_yaml::Value> = get_next_image_to_process(
            image_yaml_vec["images"].as_sequence().unwrap(),
            &ref_name_processed_vec,
        );

        assert!(
            next_image_to_process_1.unwrap()["name"].as_str().unwrap() == "base_image"
                && next_image_to_process_2.unwrap()["name"].as_str().unwrap() == "final_image"
                && next_image_to_process_3.is_none()
        );
    }
}
