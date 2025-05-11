use crate::cli::commands::apply_sat_file::utils::render_jinja2_sat_file_yaml;

/* /// Test function "get_ref_name" so it falls back to "name" field if "ref_name" is missing
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
    let next_image_to_process: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    let image_ref = get_image_name_or_ref_name_to_process(&next_image_to_process.unwrap());

    assert!(image_ref == "base_image");
} */

/* /// Test function "get_next_image_to_process" in an images section is SAT file with one image with ref_name
#[test]
fn test_get_next_image_to_process_1() {
    let image_yaml_vec: serde_yaml::Value = serde_yaml::from_str(r#"images: []"#).unwrap();

    println!(
        "image yaml vec:\n{}",
        serde_yaml::to_string(&image_yaml_vec).unwrap()
    );

    let ref_name_processed_vec: Vec<String> = Vec::new();

    let next_image_to_process: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    println!(
        "next image to process:\n{}",
        serde_yaml::to_string(&next_image_to_process).unwrap()
    );

    assert!(next_image_to_process.is_none());
} */

/* #[test]
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
    let next_image_to_process: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    println!(
        "next image to process:\n{}",
        serde_yaml::to_string(&next_image_to_process).unwrap()
    );

    assert!(next_image_to_process.unwrap()["name"].as_str().unwrap() == "base_image");
} */

/* /// Test function "get_next_image_to_process" in an images section in SAT file with 2 images.
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

    let next_image_to_process_1: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    ref_name_processed_vec.push("base_cos_image".to_string());

    let next_image_to_process_2: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    assert!(
        next_image_to_process_1.unwrap()["name"].as_str().unwrap() == "base_image"
            && next_image_to_process_2.unwrap()["name"].as_str().unwrap() == "final_image"
    );
} */

/* #[test]
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

    let next_image_to_process_1: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    ref_name_processed_vec.push("base_cos_image".to_string());

    let next_image_to_process_2: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    ref_name_processed_vec.push("compute_image".to_string());

    let next_image_to_process_3: Option<serde_yaml::Value> = get_next_image_in_sat_file_to_process(
        image_yaml_vec["images"].as_sequence().unwrap(),
        &ref_name_processed_vec,
    );

    assert!(
        next_image_to_process_1.unwrap()["name"].as_str().unwrap() == "base_image"
            && next_image_to_process_2.unwrap()["name"].as_str().unwrap() == "final_image"
            && next_image_to_process_3.is_none()
    );
} */

/// Test rendering a SAT template file with the values file
#[test]
fn test_render_sat_file_yaml_template_with_yaml_values_file() {
  let sat_file_content = r#"
        name: "{{ config.name }}"
        configurations:
        - name: "{{ config.name }}-{{ config.version }}"
          layers:
          - name: ss11
            playbook: shs_cassini_install.yml
            git:
              url: https://api-gw-service-nmn.local/vcs/cray/slingshot-host-software-config-management.git
              branch: integration
          - name: cos
            playbook: site.yml
            product:
              name: cos
              version: 2.3.101
              branch: integration
          - name: cscs
            playbook: site.yml
            git:
              url: https://api-gw-service-nmn.local/vcs/cray/cscs-config-management.git
              branch: cscs-23.06.0
          - name: nomad-orchestrator
            playbook: site-client.yml
            git:
              url: https://api-gw-service-nmn.local/vcs/cray/nomad_orchestrator.git
              branch: main
        images:
        - name: zinal-nomad-{{ image.version }}
          ims:
            is_recipe: false
            id: 4bf91021-8d99-4adf-945f-46de2ff50a3d
          configuration: "{{ config.name }}-{{ config.version }}"
          configuration_group_names:
          - Compute
          - "{{ hsm.group_name }}"

        session_templates:
        - name: "{{ bos_st.name }}"
          image: zinal-image-v0.5
          configuration: "{{ config.name }}-{{ config.version }}"
          bos_parameters:
            boot_sets:
              compute:
                kernel_parameters: ip=dhcp quiet spire_join_token=${SPIRE_JOIN_TOKEN}
                node_groups:
                - "{{ hsm.group_name }}"
        "#;

  let values_file_content = r#"
        hsm:
          group_name: "zinal_cta"
        config:
          name: "test-config"
          version: "v1.0.0"
        image:
          version: "v1.0.5"
        bos_st:
          name: "deploy-cluster-action"
          version: "v1.0"
        "#;

  let var_content: Vec<String> = vec!["config.name = new-value".to_string()];

  render_jinja2_sat_file_yaml(
    &sat_file_content.to_string(),
    Some(&values_file_content.to_string()),
    Some(var_content),
  );
}

/* /// Test SAT file
/// Test image section in OLD format in SAT file
/// Result: FAIL
/// Reason: Configuration assigned to an image could not be found
#[test]
fn test_sat_file_image_section_fails_because_configuration_could_not_be_found_in_old_image_format()
{
    let cray_product_catalog = &BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              ims:
                id: my-image-id
                is_recipe: false
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &vec![];

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        id: Some("my-image-id".to_string()),
        created: None,
        name: "my-image-name".to_string(),
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    assert!(validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes
    )
    .is_err());
} */

/* /// Test SAT file
/// Test image section in OLD format in SAT file
/// Result: PASS
/// Reason: configuration assigned to image found in SAT
#[test]
fn test_old_image_format_in_sat_file_pass_because_configuration_found_in_sat() {
    let cray_product_catalog = &BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              ims:
                id: my-base-image-id
                is_recipe: false
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        id: Some("my-base-image-id".to_string()),
        created: None,
        name: "my-base-image-name".to_string(),
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    assert!(validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes
    )
    .is_ok());
} */

/* /// Test SAT file
/// Test image section in OLD format in SAT file
/// Result: PASS
/// Reason: configuration assigned to image found in CSM
#[test]
fn test_old_image_format_in_sat_file_pass_because_configuration_found_in_csm() {
    let cray_product_catalog = &BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              ims:
                id: my-base-image-id
                is_recipe: false
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &vec![];

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        id: Some("my-base-image-id".to_string()),
        created: None,
        name: "my-base-image-name".to_string(),
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![CfsConfigurationResponse {
        name: "my-configuration-name".to_string(),
        last_updated: "2023-10-04T14:15:22Z".to_string(),
        layers: vec![Layer {
            clone_url: "fake-url".to_string(),
            commit: None,
            name: "my-layer-name".to_string(),
            playbook: "my-playbook".to_string(),
            branch: None,
            source: None,
        }],
        additional_inventory: None,
    }];

    let ims_recipes = vec![];

    assert!(validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes
    )
    .is_ok());
} */

/* /// Test SAT file
/// Test image section in OLD format in SAT file
/// Result: FAIL
/// Reason: Base image id assigned to an image could not be found
#[test]
fn test_sat_file_image_section_fails_because_base_image_id_could_not_be_found() {
    let cray_product_catalog = &BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              ims:
                id: my-image-id
                is_recipe: false
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &vec![];

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        id: Some("fake".to_string()),
        created: None,
        name: "my-image-name".to_string(),
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    assert!(validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes
    )
    .is_err());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: FAIL
/// Reason: Base image recipe assigned to an image could not be found in Cray/HPE product catalog
#[test]
fn test_sat_file_image_section_fails_because_base_image_receipe_could_not_be_found_in_cray_product_catalog(
) {
    let mut cray_product_catalog = BTreeMap::<String, String>::new();
    cray_product_catalog.insert("cos".to_string(), "2.2.101:\n  configuration:\n    clone_url: https://vcs.alps.cscs.ch/vcs/cray/cos-config-management.git\n    commit: 7f71cdc5d58f7879dc431b3fd6330296dcb3f7ee\n    import_branch: cray/cos/2.2.101\n    import_date: 2022-06-01 17:57:17.019149\n    ssh_url: git@vcs.alps.cscs.ch:cray/cos-config-management.git\n  images:\n    cray-shasta-compute-sles15sp3.x86_64-2.2.38:\n      id: d737e902-c002-4269-a408-6baa0fb31b4d\n  recipes:\n    cray-shasta-compute-sles15sp3.x86_64-2.2.38:\n      id: 2ffe10da-67f5-47b7-b7fb-de25381498f0".to_string());

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                product:
                  name: cos
                  version: "fake-does-not-exists-in-cra-product-catalog"
                  type: recipe
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_err());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: FAIL
/// Reason: Base IMS recipe assigned to an image could not be found
#[test]
fn test_sat_file_image_section_fails_because_base_image_recipe_name_could_not_be_found() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: my-ims-recipe
                  type: recipe
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![RecipeGetResponse {
        id: None,
        created: None,
        link: None,
        recipe_type: "".to_string(),
        linux_distribution: "".to_string(),
        name: "fake-my-ims-recipe".to_string(),
    }];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_err());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: PASS
/// Reason: Base IMS recipe assigned to an image found in CSM
#[test]
fn test_sat_file_image_section_pass_because_base_image_recipe_name_could_not_be_found() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: my-ims-recipe-name
                  type: recipe
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![RecipeGetResponse {
        id: None,
        created: None,
        link: None,
        recipe_type: "".to_string(),
        linux_distribution: "".to_string(),
        name: "my-ims-recipe-name".to_string(),
    }];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_ok());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: FAIL
/// Reason: Base IMS image assigned to an image not found in CSM
#[test]
fn test_sat_file_image_section_fail_because_base_image_name_could_not_be_found() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: fake-my-ims-image-name
                  type: image
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        name: "my-ims-image-name".to_string(),
        id: None,
        created: None,
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_err());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: PASS
/// Reason: Base IMS image assigned to an image found in CSM
#[test]
fn test_sat_file_image_section_pass_because_base_image_name_could_not_be_found() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: my-ims-image-name
                  type: image
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        name: "my-ims-image-name".to_string(),
        id: None,
        created: None,
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_ok());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: FAIL
/// Reason: HSM groups assigned to an image are wrong
#[test]
fn test_sat_file_image_section_fail_because_hsm_groups_are_wrong() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: my-ims-image-name
                  type: image
              configuration: my-configuration-name
              configuration_group_names:
                - Compute
                - fake-tenant-a
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-configuration-name
            "#,
    )
    .unwrap();

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        name: "my-ims-image-name".to_string(),
        id: None,
        created: None,
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_err());
} */

/* /// Test SAT file
/// Test image section in NEW format in SAT file
/// Result: PASS
/// Reason: Image can miss 'configuration' section
#[test]
fn test_sat_file_image_section_pass_if_configuration_missing() {
    let cray_product_catalog = BTreeMap::<String, String>::new();

    let image_vec_in_sat_file: &Vec<serde_yaml::Value> = &serde_yaml::from_str(
        r#"
            - name: my-image-name
              base:
                ims:
                  name: my-ims-image-name
                  type: image
            "#,
    )
    .unwrap();

    let configuration_vec_in_sat_file: &Vec<serde_yaml::Value> = &vec![];

    let hsm_group_available_vec = &["tenant-a".to_string()];

    let image_vec_in_csm = vec![Image {
        name: "my-ims-image-name".to_string(),
        id: None,
        created: None,
        link: None,
        arch: None,
    }];

    let configuration_vec_in_csm = vec![];

    let ims_recipes = vec![];

    let validation_rslt: Result<(), Error> = validate_sat_file_images_section(
        image_vec_in_sat_file,
        configuration_vec_in_sat_file,
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec_in_csm,
        configuration_vec_in_csm,
        ims_recipes,
    );

    assert!(validation_rslt.is_ok());
} */
