//! SAT-entry → HSM group-name extractors.
//!
//! Pure helpers that read the HSM-group names a single SAT `images[]`
//! or `session_templates[]` entry references, so handlers can gate
//! access at the boundary via
//! [`crate::service::group::validate_hsm_group_access_many`] before
//! delegating to the backend.
//!
//! The SAT schema lives in csm-rs and is carried as
//! `serde_json::Value` end-to-end (see ARCHITECTURE.md). These
//! functions accept the same `Value` shape the handler receives over
//! the wire and read out a `Vec<String>` of group names; they make no
//! mutation, do no I/O, and stay deliberately small so the wire
//! schema can drift without breaking the helpers.
//!
//! The shapes they read mirror the csm-rs read paths exactly:
//!
//! - Image entry → `configuration_group_names: Vec<String>`
//!   (`csm-rs/src/commands/i_apply_sat_file/utils/images.rs` —
//!   `image_yaml.configuration_group_names`).
//! - Session-template entry →
//!   `bos_parameters.boot_sets.<set>.node_groups: Vec<String>`
//!   collected and deduped across every boot_set
//!   (`csm-rs/src/commands/i_apply_sat_file/utils/session_templates.rs:54-65`).

use serde_json::Value;

/// Read `configuration_group_names` from a SAT `images[]` entry.
/// Returns an empty `Vec` when the field is absent or not an array.
pub fn extract_image_groups(image: &Value) -> Vec<String> {
  image
    .get("configuration_group_names")
    .and_then(Value::as_array)
    .map(|arr| {
      arr
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
    })
    .unwrap_or_default()
}

/// Read `bos_parameters.boot_sets.*.node_groups` from a SAT
/// `session_templates[]` entry. Collects across every boot_set key
/// (e.g. `compute`, `uan`) and deduplicates so a group named in
/// multiple boot_sets is only validated once.
pub fn extract_session_template_groups(
  session_template: &Value,
) -> Vec<String> {
  let Some(boot_sets) = session_template
    .get("bos_parameters")
    .and_then(|p| p.get("boot_sets"))
    .and_then(Value::as_object)
  else {
    return Vec::new();
  };

  let mut groups: Vec<String> = boot_sets
    .values()
    .filter_map(|set| set.get("node_groups"))
    .filter_map(Value::as_array)
    .flat_map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_string))
    .collect();
  groups.sort();
  groups.dedup();
  groups
}

#[cfg(test)]
mod tests {
  use super::{extract_image_groups, extract_session_template_groups};
  use serde_json::json;

  #[test]
  fn extract_image_groups_reads_configuration_group_names() {
    let image = json!({
      "name": "img-v1",
      "configuration": "cfg-v1",
      "configuration_group_names": ["compute", "uan"],
    });
    assert_eq!(extract_image_groups(&image), vec!["compute", "uan"]);
  }

  #[test]
  fn extract_image_groups_empty_when_field_absent() {
    let image = json!({ "name": "img-v1", "configuration": "cfg-v1" });
    assert!(extract_image_groups(&image).is_empty());
  }

  #[test]
  fn extract_image_groups_empty_when_field_is_not_array() {
    let image = json!({
      "name": "img-v1",
      "configuration_group_names": "compute",
    });
    assert!(extract_image_groups(&image).is_empty());
  }

  #[test]
  fn extract_session_template_groups_reads_all_boot_sets() {
    let template = json!({
      "name": "st-1",
      "bos_parameters": {
        "boot_sets": {
          "compute": { "node_groups": ["compute", "shared"] },
          "uan":     { "node_groups": ["uan",     "shared"] },
        }
      }
    });
    let groups = extract_session_template_groups(&template);
    assert_eq!(groups, vec!["compute", "shared", "uan"]);
  }

  #[test]
  fn extract_session_template_groups_empty_when_bos_parameters_missing() {
    let template = json!({ "name": "st-1" });
    assert!(extract_session_template_groups(&template).is_empty());
  }

  #[test]
  fn extract_session_template_groups_empty_when_boot_sets_missing() {
    let template = json!({ "name": "st-1", "bos_parameters": {} });
    assert!(extract_session_template_groups(&template).is_empty());
  }

  #[test]
  fn extract_session_template_groups_skips_boot_sets_without_node_groups() {
    let template = json!({
      "name": "st-1",
      "bos_parameters": {
        "boot_sets": {
          "compute": { "node_groups": ["compute"] },
          "uan":     { "kernel": "linux" }
        }
      }
    });
    assert_eq!(
      extract_session_template_groups(&template),
      vec!["compute"]
    );
  }
}
