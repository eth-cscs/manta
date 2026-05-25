use super::*;
use serde_json::json;
use serde_yaml::Value;

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

// ── apply_sat_file_filters ──

#[test]
fn filter_image_only_drops_session_templates_and_prunes_configurations() {
  let mut sat = json!({
      "configurations": [
          { "name": "cfg-used", "layers": [] },
          { "name": "cfg-unused", "layers": [] },
      ],
      "images": [
          { "name": "img1", "configuration": "cfg-used" },
      ],
      "session_templates": [
          { "name": "st1", "image": { "image_ref": "img1" }, "configuration": "cfg-used" },
      ],
      "hardware": [{ "pattern": "x" }],
  });

  apply_sat_file_filters(&mut sat, true, false).unwrap();

  assert!(sat.get("session_templates").is_none());
  assert!(sat.get("hardware").is_none());
  let configs = sat.get("configurations").unwrap().as_array().unwrap();
  assert_eq!(configs.len(), 1);
  assert_eq!(configs[0]["name"], "cfg-used");
}

#[test]
fn filter_session_template_only_retains_referenced_images_and_drops_unreferenced() {
  let mut sat = json!({
      "configurations": [
          { "name": "cfg-st" },
          { "name": "cfg-img-only" },
      ],
      "images": [
          { "name": "used-image", "configuration": "cfg-img-only" },
          { "name": "unused-image" },
      ],
      "session_templates": [
          { "name": "st1", "image": { "image_ref": "used-image" }, "configuration": "cfg-st" },
      ],
  });

  apply_sat_file_filters(&mut sat, false, true).unwrap();

  let images = sat.get("images").unwrap().as_array().unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0]["name"], "used-image");

  // Both configurations are kept: cfg-img-only via the surviving image,
  // cfg-st via the session template.
  let configs = sat.get("configurations").unwrap().as_array().unwrap();
  assert_eq!(configs.len(), 2);
}

#[test]
fn filter_session_template_only_drops_images_section_when_no_match() {
  let mut sat = json!({
      "configurations": [{ "name": "cfg-st" }],
      "images": [{ "name": "img-not-referenced" }],
      "session_templates": [
          { "name": "st1", "image": { "ims": { "id": "abc-123" } }, "configuration": "cfg-st" },
      ],
  });

  apply_sat_file_filters(&mut sat, false, true).unwrap();

  // No image survives the retain, so the whole section goes.
  assert!(sat.get("images").is_none());
  // Configuration remains because the session_template references it.
  let configs = sat.get("configurations").unwrap().as_array().unwrap();
  assert_eq!(configs.len(), 1);
}

#[test]
fn filter_session_template_only_matches_ims_name_variant() {
  // image: { ims: { name: "..." } } form should also retain the image.
  let mut sat = json!({
      "images": [{ "name": "ims-name-target" }],
      "session_templates": [
          { "name": "st1", "image": { "ims": { "name": "ims-name-target" } }, "configuration": "cfg" },
      ],
      "configurations": [{ "name": "cfg" }],
  });

  apply_sat_file_filters(&mut sat, false, true).unwrap();

  let images = sat.get("images").unwrap().as_array().unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0]["name"], "ims-name-target");
}

#[test]
fn filter_neither_flag_is_noop() {
  let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "images": [{ "name": "img1" }],
      "session_templates": [
          { "name": "st1", "image": { "image_ref": "img1" }, "configuration": "cfg1" },
      ],
  });
  let before = sat.clone();

  apply_sat_file_filters(&mut sat, false, false).unwrap();

  assert_eq!(sat, before);
}

#[test]
fn filter_image_only_errors_when_images_missing() {
  let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "session_templates": [],
  });
  let err = apply_sat_file_filters(&mut sat, true, false).unwrap_err();
  assert!(err.to_string().contains("'images' section missing"));
}

#[test]
fn filter_session_template_only_errors_when_section_missing() {
  let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "images": [],
  });
  let err = apply_sat_file_filters(&mut sat, false, true).unwrap_err();
  assert!(err.to_string().contains("'session_templates' section"));
}

#[test]
fn filter_errors_when_root_is_not_a_mapping() {
  let mut sat = json!([1, 2, 3]);
  let err = apply_sat_file_filters(&mut sat, true, false).unwrap_err();
  assert!(err.to_string().contains("not a YAML/JSON mapping"));
}
