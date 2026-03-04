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

  render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    Some(&var_content),
  )
  .unwrap();
}
