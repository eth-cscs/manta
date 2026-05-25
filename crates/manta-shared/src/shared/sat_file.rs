//! Jinja2 rendering helpers for SAT (System Admin Toolkit) template files
//! and a structure-aware filter that operates on the parsed
//! [`serde_json::Value`].
//!
//! The canonical SAT-file schema lives in csm-rs (which parses the value
//! into typed structs during apply). The CLI never carries the typed
//! schema — it only needs to:
//!
//! 1. Render Jinja2 templates with a values file + `--var` overrides
//!    (the renderer takes parsed YAML as Jinja context for the values
//!    file but produces a string for the SAT file content).
//! 2. Parse the rendered SAT string into a `serde_json::Value` so the
//!    server can forward it verbatim.
//! 3. Apply `--image-only` / `--sessiontemplate-only` filters by
//!    walking the Value (drop top-level sections plus prune unreferenced
//!    configurations/images) before sending.
//!
//! Both `--image-only` and `--sessiontemplate-only` preserve the
//! historical CLI semantics:
//! - `--image-only`: drops `session_templates` + `hardware`; retains
//!   only configurations referenced by the remaining images.
//! - `--sessiontemplate-only`: drops `hardware`; retains images only
//!   if named in a session_template; drops the `images` section
//!   entirely if no image survives; retains only configurations
//!   referenced by surviving images or session_templates.
//!
//! The walk navigates a small set of field names
//! (`configurations`, `images`, `session_templates`, `hardware`,
//! `name`, `configuration`, `image`, `image_ref`, `ims`) — no struct
//! schema is embedded here.

use std::collections::HashSet;

use serde_json::Value as JsonValue;
use serde_yaml::{Mapping, Value};

use crate::common::error::MantaError as Error;

/// Apply `--image-only` / `--sessiontemplate-only` filters to a parsed
/// SAT file in-place.
///
/// Mirrors the prune-by-reference semantics historically implemented by
/// the typed `SatFile::filter` method, but operates on
/// [`serde_json::Value`] so the CLI does not need to embed the SAT
/// schema. See the module-level docs for the exact filter rules.
///
/// # Errors
///
/// - [`Error::MissingField`] when `image_only` is set but no `images`
///   section is present in the SAT file.
/// - [`Error::MissingField`] when `session_template_only` is set but no
///   `session_templates` section is present.
pub fn apply_sat_file_filters(
  sat_file: &mut JsonValue,
  image_only: bool,
  session_template_only: bool,
) -> Result<(), Error> {
  if image_only {
    let obj = sat_file.as_object_mut().ok_or_else(|| {
      Error::TemplateError(
        "SAT file root is not a YAML/JSON mapping".to_string(),
      )
    })?;

    if !obj.contains_key("images") {
      return Err(Error::MissingField(
        "'images' section missing in SAT file".to_string(),
      ));
    }

    obj.remove("session_templates");
    obj.remove("hardware");

    let referenced: HashSet<String> = obj
      .get("images")
      .and_then(JsonValue::as_array)
      .map(|imgs| {
        imgs
          .iter()
          .filter_map(|img| {
            img.get("configuration")?.as_str().map(str::to_string)
          })
          .collect()
      })
      .unwrap_or_default();

    if let Some(configs) =
      obj.get_mut("configurations").and_then(JsonValue::as_array_mut)
    {
      configs.retain(|cfg| {
        cfg
          .get("name")
          .and_then(JsonValue::as_str)
          .is_some_and(|n| referenced.contains(n))
      });
    }
  }

  if session_template_only {
    let obj = sat_file.as_object_mut().ok_or_else(|| {
      Error::TemplateError(
        "SAT file root is not a YAML/JSON mapping".to_string(),
      )
    })?;

    if !obj.contains_key("session_templates") {
      return Err(Error::MissingField(
        "'session_templates' section not defined in SAT file".to_string(),
      ));
    }

    obj.remove("hardware");

    // Names of images referenced by session_templates (either by
    // `image_ref` or by `ims.name`).
    let image_keep: HashSet<String> = obj
      .get("session_templates")
      .and_then(JsonValue::as_array)
      .map(|sts| {
        sts
          .iter()
          .filter_map(image_name_referenced_by_session_template)
          .collect()
      })
      .unwrap_or_default();

    // Retain images by name; drop the section if it ends up empty.
    let images_empty = if let Some(imgs) =
      obj.get_mut("images").and_then(JsonValue::as_array_mut)
    {
      imgs.retain(|img| {
        img
          .get("name")
          .and_then(JsonValue::as_str)
          .is_some_and(|n| image_keep.contains(n))
      });
      imgs.is_empty()
    } else {
      false
    };
    if images_empty {
      obj.remove("images");
    }

    // Configurations to keep: referenced by surviving images OR by
    // any session_template.
    let mut config_keep: HashSet<String> = HashSet::new();
    if let Some(imgs) = obj.get("images").and_then(JsonValue::as_array) {
      for img in imgs {
        if let Some(c) = img.get("configuration").and_then(JsonValue::as_str)
        {
          config_keep.insert(c.to_string());
        }
      }
    }
    if let Some(sts) =
      obj.get("session_templates").and_then(JsonValue::as_array)
    {
      for st in sts {
        if let Some(c) = st.get("configuration").and_then(JsonValue::as_str) {
          config_keep.insert(c.to_string());
        }
      }
    }

    if let Some(configs) =
      obj.get_mut("configurations").and_then(JsonValue::as_array_mut)
    {
      configs.retain(|cfg| {
        cfg
          .get("name")
          .and_then(JsonValue::as_str)
          .is_some_and(|n| config_keep.contains(n))
      });
    }
  }

  Ok(())
}

/// Extract the image name a session_template entry references, in either
/// shape:
/// - `image: { image_ref: "<name>" }`
/// - `image: { ims: { name: "<name>" } }`
///
/// Returns `None` for `image: { ims: { id: "<id>" } }` (pre-built images
/// referenced by ID — no name to filter on).
fn image_name_referenced_by_session_template(
  st: &JsonValue,
) -> Option<String> {
  let image = st.get("image")?;
  if let Some(name) = image.get("image_ref").and_then(JsonValue::as_str) {
    return Some(name.to_string());
  }
  image
    .get("ims")
    .and_then(|ims| ims.get("name"))
    .and_then(JsonValue::as_str)
    .map(str::to_string)
}

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
