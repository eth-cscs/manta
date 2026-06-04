//! Jinja2 rendering helpers for SAT (System Admin Toolkit) template
//! files.
//!
//! The canonical SAT-file schema lives in csm-rs (which parses the value
//! into typed structs during apply). This module only handles the
//! pre-parse step the CLI needs:
//!
//! 1. Render Jinja2 templates with a values file + `--var` overrides
//!    (the renderer takes parsed YAML as Jinja context for the values
//!    file but produces a string for the SAT file content).
//! 2. After rendering, the CLI parses the result into a
//!    `serde_json::Value` itself.
//!
//! The `--image-only` / `--sessiontemplate-only` filters used to live
//! here too but now sit inside `build_plan` in
//! `manta-cli/src/commands/apply/sat_file/plan.rs`, where they share
//! the SAT-file walk that builds the execution plan.

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
mod tests;
