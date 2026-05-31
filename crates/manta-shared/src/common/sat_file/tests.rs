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
