use crate::cli::common::sat_file::render_jinja2_sat_file_yaml;
use serde_yaml::Value;

use crate::cli::commands::apply_sat_file::plan::{SatElement, build_plan};

/// Render a SAT template + values file (with a CLI `--var` override) and
/// confirm the rendered YAML string substitutes variables from all
/// three sources.
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

  let rendered = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    Some(&var_content),
  )
  .unwrap();

  // Parse the rendered string so assertions can navigate the structure.
  let parsed: Value = serde_yaml::from_str(&rendered).unwrap();

  // Verify CLI var override took effect (config.name = "new-value" instead of "test-config")
  let name = parsed.get("name").unwrap().as_str().unwrap();
  assert_eq!(name, "new-value", "CLI --var should override values file");

  // Verify configuration name uses the overridden config.name
  let configs = parsed.get("configurations").unwrap().as_sequence().unwrap();
  assert_eq!(configs.len(), 1);
  let config_name = configs[0].get("name").unwrap().as_str().unwrap();
  assert_eq!(config_name, "new-value-v1.0.0");

  // Verify image name uses value from values file
  let images = parsed.get("images").unwrap().as_sequence().unwrap();
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
  let templates = parsed
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

  let rendered =
    render_jinja2_sat_file_yaml(sat_file_content, None, None).unwrap();
  let parsed: Value = serde_yaml::from_str(&rendered).unwrap();

  assert_eq!(parsed.get("name").unwrap().as_str().unwrap(), "my-config");
  let configs = parsed.get("configurations").unwrap().as_sequence().unwrap();
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

  let rendered = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    None,
  )
  .unwrap();
  let parsed: Value = serde_yaml::from_str(&rendered).unwrap();

  assert_eq!(parsed.get("name").unwrap().as_str().unwrap(), "my-app");
  assert_eq!(parsed.get("version").unwrap().as_str().unwrap(), "2.0");
}

/// End-to-end client-side pipeline: render Jinja2, parse to
/// `serde_json::Value`, apply `image_only=true`, and confirm the
/// session_templates section was stripped and unreferenced
/// configurations were pruned before the value would be sent.
#[test]
fn test_client_side_pipeline_with_image_only_filter() {
  let sat_file_content = r#"
configurations:
- name: cfg-{{ app.version }}
  layers:
    - name: layer1
      git:
        url: https://example.com/repo.git
        branch: main
- name: cfg-unused-{{ app.version }}
  layers:
    - name: layer-unused
      git:
        url: https://example.com/repo.git
        branch: main
images:
- name: img-{{ app.version }}
  ims:
    is_recipe: false
    id: abc-123
  configuration: cfg-{{ app.version }}
session_templates:
- name: st-{{ app.version }}
  image:
    image_ref: img-{{ app.version }}
  configuration: cfg-{{ app.version }}
  bos_parameters:
    boot_sets:
      compute:
        node_groups:
          - group1
"#;
  let values_file_content = r#"
app:
  version: v1
"#;

  let rendered = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    None,
  )
  .expect("render");
  let mut sat: serde_json::Value =
    serde_yaml::from_str(&rendered).expect("parse to Value");

  let plan = build_plan(&mut sat, true, false).expect("build_plan image_only");

  assert!(
    sat.get("session_templates").is_none(),
    "image_only filter should drop session_templates"
  );
  let images = sat.get("images").unwrap().as_array().unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0]["name"], "img-v1");
  // Only the referenced configuration survives the prune.
  let configs = sat.get("configurations").unwrap().as_array().unwrap();
  assert_eq!(configs.len(), 1);
  assert_eq!(configs[0]["name"], "cfg-v1");

  // Plan reflects the same: one configuration, one image, no session_template.
  assert_eq!(plan.len(), 2);
  assert!(matches!(plan[0], SatElement::Configuration(_)));
  assert!(matches!(plan[1], SatElement::Image(_)));
}
