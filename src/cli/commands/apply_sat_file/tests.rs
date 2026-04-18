use crate::cli::commands::apply_sat_file::utils::render_jinja2_sat_file_yaml;

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

  let result = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    Some(&var_content),
  )
  .unwrap();

  // Verify CLI var override took effect (config.name = "new-value" instead of "test-config")
  let name = result.get("name").unwrap().as_str().unwrap();
  assert_eq!(name, "new-value", "CLI --var should override values file");

  // Verify configuration name uses the overridden config.name
  let configs = result.get("configurations").unwrap().as_sequence().unwrap();
  assert_eq!(configs.len(), 1);
  let config_name = configs[0].get("name").unwrap().as_str().unwrap();
  assert_eq!(config_name, "new-value-v1.0.0");

  // Verify image name uses value from values file
  let images = result.get("images").unwrap().as_sequence().unwrap();
  let image_name = images[0].get("name").unwrap().as_str().unwrap();
  assert_eq!(image_name, "zinal-nomad-v1.0.5");

  // Verify HSM group name was substituted
  let image_groups = images[0]
    .get("configuration_group_names")
    .unwrap()
    .as_sequence()
    .unwrap();
  assert_eq!(image_groups[1].as_str().unwrap(), "zinal_cta");

  // Verify session template name
  let templates = result
    .get("session_templates")
    .unwrap()
    .as_sequence()
    .unwrap();
  let st_name = templates[0].get("name").unwrap().as_str().unwrap();
  assert_eq!(st_name, "deploy-cluster-action");

  // Verify session template configuration uses overridden name
  let st_config = templates[0].get("configuration").unwrap().as_str().unwrap();
  assert_eq!(st_config, "new-value-v1.0.0");
}

/// Test rendering without a values file fails when template has variables
#[test]
fn test_render_sat_file_without_values_file() {
  let sat_file_content = r#"
        name: "{{ config.name }}"
        "#;

  let result = render_jinja2_sat_file_yaml(sat_file_content, None, None);

  // Should fail because config.name is undefined
  assert!(
    result.is_err(),
    "Rendering with undefined variables should fail"
  );
}

/// Test rendering a SAT file with no template variables (plain YAML)
#[test]
fn test_render_plain_sat_file_no_variables() {
  let sat_file_content = r#"
        name: "my-config"
        configurations:
        - name: "static-config"
          layers:
          - name: layer1
            playbook: site.yml
            git:
              url: https://example.com/repo.git
              branch: main
        "#;

  let result =
    render_jinja2_sat_file_yaml(sat_file_content, None, None).unwrap();

  assert_eq!(result.get("name").unwrap().as_str().unwrap(), "my-config");
  let configs = result.get("configurations").unwrap().as_sequence().unwrap();
  assert_eq!(
    configs[0].get("name").unwrap().as_str().unwrap(),
    "static-config"
  );
}

/// Test that values file without CLI overrides works correctly
#[test]
fn test_render_sat_file_with_values_no_overrides() {
  let sat_file_content = r#"
        name: "{{ app.name }}"
        version: "{{ app.version }}"
        "#;

  let values_file_content = r#"
        app:
          name: "my-app"
          version: "2.0"
        "#;

  let result = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    None,
  )
  .unwrap();

  assert_eq!(result.get("name").unwrap().as_str().unwrap(), "my-app");
  assert_eq!(result.get("version").unwrap().as_str().unwrap(), "2.0");
}
