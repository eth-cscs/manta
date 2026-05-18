use super::*;
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

// ── SatFile::filter ──

#[test]
fn filter_image_only_removes_session_templates() {
  let yaml = r#"
configurations:
- name: cfg-used
  layers:
    - name: layer1
      git:
        url: https://example.com/repo.git
        branch: main
- name: cfg-unused
  layers:
    - name: layer2
      git:
        url: https://example.com/repo.git
        branch: main
images:
- name: img1
  ims:
    is_recipe: false
    id: abc-123
  configuration: cfg-used
session_templates:
- name: st1
  image:
    image_ref: img1
  configuration: cfg-used
  bos_parameters:
    boot_sets:
      compute:
        node_groups:
          - group1
"#;
  let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
  sat.filter(true, false).unwrap();
  assert!(sat.session_templates.is_none());
  // Only cfg-used is kept
  let configs = sat.configurations.unwrap();
  assert_eq!(configs.len(), 1);
  assert_eq!(configs[0].name, "cfg-used");
}

#[test]
fn filter_session_template_only_removes_unused_images() {
  let yaml = r#"
configurations:
- name: cfg-st
  layers:
    - name: layer1
      git:
        url: https://example.com/repo.git
        branch: main
- name: cfg-img-only
  layers:
    - name: layer2
      git:
        url: https://example.com/repo.git
        branch: main
images:
- name: used-image
  ims:
    is_recipe: false
    id: abc-123
  configuration: cfg-img-only
- name: unused-image
  ims:
    is_recipe: false
    id: def-456
session_templates:
- name: st1
  image:
    image_ref: used-image
  configuration: cfg-st
  bos_parameters:
    boot_sets:
      compute:
        node_groups:
          - group1
"#;
  let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
  sat.filter(false, true).unwrap();
  // Only used-image should remain
  let images = sat.images.unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0].name, "used-image");
}

#[test]
fn filter_neither_flag_is_noop() {
  let yaml = r#"
configurations:
- name: cfg1
  layers:
    - name: layer1
      git:
        url: https://example.com/repo.git
        branch: main
images:
- name: img1
  ims:
    is_recipe: false
    id: abc-123
session_templates:
- name: st1
  image:
    image_ref: img1
  configuration: cfg1
  bos_parameters:
    boot_sets:
      compute:
        node_groups:
          - group1
"#;
  let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
  sat.filter(false, false).unwrap();
  assert!(sat.images.is_some());
  assert!(sat.session_templates.is_some());
  assert!(sat.configurations.is_some());
}
