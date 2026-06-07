use super::*;

// ---- validate_xname_format ----

#[test]
fn valid_xname() {
  assert!(validate_xname_format("x1000c0s0b0n0"));
}

#[test]
fn valid_xname_max_values() {
  assert!(validate_xname_format("x9999c7s64b1n7"));
}

#[test]
fn invalid_xname_missing_prefix() {
  assert!(!validate_xname_format("1000c0s0b0n0"));
}

#[test]
fn invalid_xname_bad_cabinet() {
  assert!(!validate_xname_format("x100c0s0b0n0"));
}

#[test]
fn invalid_xname_slot_too_high() {
  assert!(!validate_xname_format("x1000c0s65b0n0"));
}

#[test]
fn invalid_xname_board_too_high() {
  assert!(!validate_xname_format("x1000c0s0b2n0"));
}

#[test]
fn invalid_xname_node_too_high() {
  assert!(!validate_xname_format("x1000c0s0b0n8"));
}

#[test]
fn invalid_xname_chassis_too_high() {
  assert!(!validate_xname_format("x1000c8s0b0n0"));
}

#[test]
fn invalid_xname_empty() {
  assert!(!validate_xname_format(""));
}

#[test]
fn invalid_xname_garbage() {
  assert!(!validate_xname_format("not-an-xname"));
}

// ---- validate_nid_format ----

#[test]
fn valid_nid() {
  assert!(validate_nid_format("nid000001"));
}

#[test]
fn valid_nid_all_zeros() {
  assert!(validate_nid_format("nid000000"));
}

#[test]
fn invalid_nid_too_short() {
  assert!(!validate_nid_format("nid001"));
}

#[test]
fn invalid_nid_too_long() {
  assert!(!validate_nid_format("nid0000001"));
}

#[test]
fn invalid_nid_missing_prefix() {
  assert!(!validate_nid_format("000000001"));
}

#[test]
fn invalid_nid_non_numeric() {
  assert!(!validate_nid_format("nid00000a"));
}

#[test]
fn invalid_nid_uppercase() {
  // validate_nid_format lowercases for starts_with check
  // but strip_prefix("nid") on the original string fails
  // for uppercase input, so uppercase NIDs are rejected
  assert!(!validate_nid_format("NID000001"));
}

// ---- get_short_nid ----

#[test]
fn short_nid_valid() {
  assert_eq!(get_short_nid("nid000001").unwrap(), 1);
}

#[test]
fn short_nid_larger_number() {
  assert_eq!(get_short_nid("nid001234").unwrap(), 1234);
}

#[test]
fn short_nid_zero() {
  assert_eq!(get_short_nid("nid000000").unwrap(), 0);
}

#[test]
fn short_nid_wrong_length() {
  assert!(get_short_nid("nid001").is_err());
}

#[test]
fn short_nid_no_prefix() {
  assert!(get_short_nid("xxx000001").is_err());
}

// ---- helper ----

/// Build a minimal `Component` with only `id` and `nid` populated;
/// every other field is `None`.
fn make_component(id: &str, nid: Option<usize>) -> Component {
  Component {
    id: Some(id.to_string()),
    r#type: None,
    state: None,
    flag: None,
    enabled: None,
    software_status: None,
    role: None,
    sub_role: None,
    nid,
    subtype: None,
    net_type: None,
    arch: None,
    class: None,
    reservation_disabled: None,
    locked: None,
  }
}

// ---- get_xname_from_nid_hostlist ----

#[tokio::test]
async fn nid_hostlist_matching_nids() {
  let metadata = vec![
    make_component("x1000c0s0b0n0", Some(1)),
    make_component("x1000c0s1b0n0", Some(2)),
    make_component("x1000c0s2b0n0", Some(3)),
  ];
  let nids = vec!["nid000001".to_string(), "nid000003".to_string()];

  let result = get_xname_from_nid_hostlist(&nids, &metadata).unwrap();
  assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s2b0n0"]);
}

#[tokio::test]
async fn nid_hostlist_no_match() {
  let metadata = vec![make_component("x1000c0s0b0n0", Some(1))];
  let nids = vec!["nid000099".to_string()];

  let result = get_xname_from_nid_hostlist(&nids, &metadata).unwrap();
  assert!(result.is_empty());
}

#[tokio::test]
async fn nid_hostlist_empty_inputs() {
  let result = get_xname_from_nid_hostlist(&[], &[]).unwrap();
  assert!(result.is_empty());
}

// ---- get_xname_from_xname_hostlist ----

#[tokio::test]
async fn xname_hostlist_matching_xnames() {
  let metadata = vec![
    make_component("x1000c0s0b0n0", Some(1)),
    make_component("x1000c0s1b0n0", Some(2)),
    make_component("x1000c0s2b0n0", Some(3)),
  ];
  let xnames = vec!["x1000c0s0b0n0".to_string(), "x1000c0s2b0n0".to_string()];

  let result = get_xname_from_xname_hostlist(&xnames, &metadata).unwrap();
  assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s2b0n0"]);
}

#[tokio::test]
async fn xname_hostlist_no_match() {
  let metadata = vec![make_component("x1000c0s0b0n0", Some(1))];
  let xnames = vec!["x9999c0s0b0n0".to_string()];

  let result = get_xname_from_xname_hostlist(&xnames, &metadata).unwrap();
  assert!(result.is_empty());
}

#[tokio::test]
async fn xname_hostlist_empty_inputs() {
  let result = get_xname_from_xname_hostlist(&[], &[]).unwrap();
  assert!(result.is_empty());
}

// ---- from_hosts_expression_to_xname_vec ----

#[tokio::test]
async fn hosts_expression_nid_list() {
  let metadata = vec![
    make_component("x1000c0s0b0n0", Some(1)),
    make_component("x1000c0s1b0n0", Some(2)),
    make_component("x1000c0s2b0n0", Some(3)),
  ];
  // Comma-separated NID hostlist
  let result =
    from_hosts_expression_to_xname_vec("nid000001,nid000002", false, metadata)
      .unwrap();
  assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s1b0n0"]);
}

#[tokio::test]
async fn hosts_expression_xname_list() {
  let metadata = vec![
    make_component("x1000c0s0b0n0", Some(1)),
    make_component("x1000c0s1b0n0", Some(2)),
  ];
  let result = from_hosts_expression_to_xname_vec(
    "x1000c0s0b0n0,x1000c0s1b0n0",
    false,
    metadata,
  )
  .unwrap();
  assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s1b0n0"]);
}

#[tokio::test]
async fn hosts_expression_invalid_input() {
  let metadata = vec![make_component("x1000c0s0b0n0", Some(1))];
  // "foobar" is neither a valid NID nor xname
  let result =
    from_hosts_expression_to_xname_vec("foobar", false, metadata);
  assert!(result.is_err());
}

#[tokio::test]
async fn hosts_expression_nid_no_metadata_match_returns_error() {
  // All NIDs are valid but none match the metadata -> empty -> error
  let metadata = vec![make_component("x1000c0s0b0n0", Some(99))];
  let result =
    from_hosts_expression_to_xname_vec("nid000001", false, metadata);
  assert!(result.is_err());
}

#[tokio::test]
async fn hosts_expression_include_siblings() {
  // Two nodes on the same blade (x1000c0s0b0), one on a different blade
  let metadata = vec![
    make_component("x1000c0s0b0n0", Some(1)),
    make_component("x1000c0s0b0n1", Some(2)), // sibling of n0
    make_component("x1000c0s1b0n0", Some(3)), // different blade
  ];
  // Request only nid000001 but include siblings
  let mut result =
    from_hosts_expression_to_xname_vec("nid000001", true, metadata).unwrap();
  result.sort();
  assert_eq!(result, vec!["x1000c0s0b0n0", "x1000c0s0b0n1"]);
}

// ── validate_nid_format_vec ──

#[test]
fn validate_nid_format_vec_all_valid() {
  let nids = vec!["nid000001".to_string(), "nid000099".to_string()];
  assert!(validate_nid_format_vec(&nids));
}

#[test]
fn validate_nid_format_vec_one_invalid() {
  let nids = vec!["nid000001".to_string(), "x1000c0s0b0n0".to_string()];
  assert!(!validate_nid_format_vec(&nids));
}

#[test]
fn validate_nid_format_vec_empty() {
  let nids: Vec<String> = vec![];
  assert!(
    validate_nid_format_vec(&nids),
    "empty vec should return true (vacuous truth)"
  );
}

#[test]
fn validate_nid_format_vec_all_invalid() {
  let nids = vec!["bad".to_string(), "worse".to_string()];
  assert!(!validate_nid_format_vec(&nids));
}

// ── validate_xname_format_vec ──

#[test]
fn validate_xname_format_vec_all_valid() {
  let xnames = vec!["x1000c0s0b0n0".to_string(), "x9999c7s7b1n1".to_string()];
  assert!(validate_xname_format_vec(&xnames));
}

#[test]
fn validate_xname_format_vec_one_invalid() {
  let xnames = vec!["x1000c0s0b0n0".to_string(), "nid000001".to_string()];
  assert!(!validate_xname_format_vec(&xnames));
}

#[test]
fn validate_xname_format_vec_empty() {
  let xnames: Vec<String> = vec![];
  assert!(
    validate_xname_format_vec(&xnames),
    "empty vec should return true (vacuous truth)"
  );
}

#[test]
fn validate_xname_format_vec_all_invalid() {
  let xnames = vec!["garbage".to_string(), "not_xname".to_string()];
  assert!(!validate_xname_format_vec(&xnames));
}
