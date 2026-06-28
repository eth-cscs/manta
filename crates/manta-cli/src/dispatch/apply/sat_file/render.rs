//! Jinja2 rendering for SAT (System Admin Toolkit) template files —
//! the first step of the `manta apply sat-file` pipeline.
//!
//! The canonical SAT-file schema lives in csm-rs (which parses the value
//! into typed structs during apply). This module only handles the
//! pre-parse step the CLI needs:
//!
//! 1. Render Jinja2 templates with a values file + `--var` overrides
//!    (the renderer takes parsed YAML as Jinja context for the values
//!    file but produces a string for the SAT file content).
//! 2. After rendering, the caller parses the result into a
//!    `serde_json::Value` itself.
//!
//! The `--image-only` / `--sessiontemplate-only` filters live inside
//! `build_plan` in [`super::plan`], where they share the SAT-file walk
//! that builds the execution plan.

use serde_yaml::{Mapping, Value};

use manta_shared::common::error::MantaError as Error;

/// Merges two `serde_yaml::Value`s into a single `serde_yaml::Value`.
/// `merge` values override `base` values when keys collide; sequences
/// concatenate. Used to layer CLI `--var` overrides on top of a values
/// file during Jinja rendering.
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

/// Convert a String dot notation expression into a `serde_yaml::Value`.
/// eg:
/// dot notation input like:
/// ```text
/// key_1.key_2.key_3=1
/// ```
/// would result in a `serde_yaml::Value` equivalent to:
/// ```text
/// key_1
///   key_2
///     key_3: 1
/// ```
fn dot_notation_to_yaml(dot_notation: &str) -> Result<Value, Error> {
  let parts: Vec<&str> = dot_notation.split('=').collect();
  if parts.len() != 2 {
    return Err(Error::InvalidPattern("Invalid format".to_string()));
  }

  let keys: Vec<&str> = parts[0].trim().split('.').collect();
  let value_str = parts[1].trim().trim_matches('"');
  let value: Value = Value::String(value_str.to_string());

  let mut root = Value::Mapping(Mapping::new());
  let mut current_level = &mut root;

  for (i, &key) in keys.iter().enumerate() {
    if i == keys.len() - 1 {
      if let Value::Mapping(map) = current_level {
        map.insert(Value::String(key.to_string()), value.clone());
      }
    } else {
      let next_level = if let Value::Mapping(map) = current_level {
        if map.contains_key(Value::String(key.to_string())) {
          map.get_mut(Value::String(key.to_string())).ok_or_else(|| {
            Error::TemplateError(
              "Failed to get mutable reference to existing YAML map entry"
                .to_string(),
            )
          })?
        } else {
          map.insert(
            Value::String(key.to_string()),
            Value::Mapping(Mapping::new()),
          );
          map.get_mut(Value::String(key.to_string())).ok_or_else(|| {
            Error::TemplateError(
              "Failed to get mutable reference to newly inserted YAML map entry"
                .to_string(),
            )
          })?
        }
      } else {
        return Err(Error::TemplateError(
          "Unexpected structure encountered".to_string(),
        ));
      };
      current_level = next_level;
    }
  }

  Ok(root)
}

/// Render a SAT file as a Jinja2 template, optionally merging a values
/// file and CLI-provided overrides in dot notation. Returns the
/// rendered SAT YAML as a string — callers parse it into the structured
/// value they need (CLI parses to [`serde_json::Value`]).
///
/// Precedence on variable conflicts (highest wins): CLI `--var`
/// overrides, then the values file, then the SAT file itself. Strict
/// undefined-variable behaviour is enabled — referencing an unset
/// variable returns an error rather than rendering an empty string.
///
/// # Errors
///
/// Returns a [`MantaError`](manta_shared::common::error::MantaError)
/// when the Jinja2 environment fails to configure, when the values
/// file is not valid YAML, when a CLI `--var` string is not in
/// `key.path=value` form, when merging the CLI overrides fails, or
/// when either render step encounters an undefined variable or
/// malformed template.
pub fn render_jinja2_sat_file_yaml(
  sat_file_content: &str,
  values_file_content_opt: Option<&str>,
  value_cli_vec_opt: Option<&[String]>,
) -> Result<String, Error> {
  let mut env = minijinja::Environment::new();
  // Set/enable debug in order to force minijinja to print debug error messages which are more
  // descriptive. Eg https://github.com/mitsuhiko/minijinja/blob/main/examples/error/src/main.rs#L4-L5
  env.set_debug(true);
  // Set lines starting with `#` as comments
  env.set_syntax(
    minijinja::syntax::SyntaxConfig::builder()
      .line_comment_prefix("#")
      .build()
      .map_err(|e| {
        Error::TemplateError(format!(
          "Failed to build jinja2 syntax config: {e}"
        ))
      })?,
  );
  // Set 'String' as undefined behaviour meaning, missing values won't pass the template
  // rendering
  env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

  // Render session values file
  let mut values_file_yaml: Value = if let Some(values_file_content) =
    values_file_content_opt
  {
    tracing::info!(
      "'Session vars' file provided. Going to process SAT file as a jinja template."
    );
    tracing::info!("Expand variables in 'session vars' file");
    let values_file_yaml: Value = serde_yaml::from_str(values_file_content)?;
    let values_file_rendered = env
      .render_str(values_file_content, values_file_yaml)
      .map_err(|e| {
        Error::TemplateError(format!("Error parsing values file to YAML: {e}"))
      })?;
    serde_yaml::from_str(&values_file_rendered)?
  } else {
    serde_yaml::from_str(sat_file_content)?
  };

  // Convert variable values sent by cli argument from dot notation to yaml format
  tracing::debug!(
    "Convert variable values sent by cli argument from dot notation to yaml format"
  );
  if let Some(value_option_vec) = value_cli_vec_opt {
    for value_option in value_option_vec {
      let cli_var_context_yaml = dot_notation_to_yaml(value_option)?;

      values_file_yaml =
        merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).ok_or_else(
          || {
            Error::TemplateError(
              "Failed to merge CLI variable values into \
             SAT file YAML"
                .to_string(),
            )
          },
        )?;
    }
  }

  // render sat template file
  tracing::info!("Expand variables in 'SAT file'");
  let sat_file_rendered = env
    .render_str(sat_file_content, values_file_yaml)
    .map_err(|e| {
      Error::TemplateError(format!("Failed to render SAT file template: {e}"))
    })?;

  // Disable debug
  env.set_debug(false);

  Ok(sat_file_rendered)
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── merge_yaml ──

  #[test]
  fn merge_yaml_scalars_overwrite() {
    let base = Value::String("old".into());
    let merge = Value::String("new".into());
    let result = merge_yaml(base, merge).unwrap();
    assert_eq!(result, Value::String("new".into()));
  }

  #[test]
  fn merge_yaml_maps_deep_merge() {
    let base: Value = serde_yaml::from_str("a:\n  b: 1\n  c: 2").unwrap();
    let merge: Value = serde_yaml::from_str("a:\n  b: 99\n  d: 3").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    // b is overwritten, c is preserved, d is added
    let a = result.get("a").unwrap();
    assert_eq!(a.get("b").unwrap().as_u64(), Some(99));
    assert_eq!(a.get("c").unwrap().as_u64(), Some(2));
    assert_eq!(a.get("d").unwrap().as_u64(), Some(3));
  }

  #[test]
  fn merge_yaml_sequences_concatenated() {
    let base: Value = serde_yaml::from_str("[1, 2]").unwrap();
    let merge: Value = serde_yaml::from_str("[3, 4]").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    let seq = result.as_sequence().unwrap();
    assert_eq!(seq.len(), 4);
  }

  #[test]
  fn merge_yaml_adds_new_top_level_keys() {
    let base: Value = serde_yaml::from_str("x: 1").unwrap();
    let merge: Value = serde_yaml::from_str("y: 2").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    assert_eq!(result.get("x").unwrap().as_u64(), Some(1));
    assert_eq!(result.get("y").unwrap().as_u64(), Some(2));
  }

  // ── dot_notation_to_yaml ──

  #[test]
  fn dot_notation_single_key() {
    let result = dot_notation_to_yaml("key=value").unwrap();
    assert_eq!(result.get("key").unwrap().as_str(), Some("value"));
  }

  #[test]
  fn dot_notation_nested_keys() {
    let result = dot_notation_to_yaml("a.b.c=hello").unwrap();
    let a = result.get("a").unwrap();
    let b = a.get("b").unwrap();
    assert_eq!(b.get("c").unwrap().as_str(), Some("hello"));
  }

  #[test]
  fn dot_notation_strips_quotes_from_value() {
    let result = dot_notation_to_yaml("key=\"quoted\"").unwrap();
    assert_eq!(result.get("key").unwrap().as_str(), Some("quoted"));
  }

  #[test]
  fn dot_notation_invalid_format_no_equals() {
    assert!(dot_notation_to_yaml("no_equals_sign").is_err());
  }

  #[test]
  fn dot_notation_multiple_equals_rejected() {
    assert!(dot_notation_to_yaml("a=b=c").is_err());
  }

  // ── render_jinja2_sat_file_yaml ──

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
    let st_config =
      templates[0].get("configuration").unwrap().as_str().unwrap();
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
}
