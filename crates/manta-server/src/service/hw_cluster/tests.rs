use super::pin_unpin::{parse_hw_pattern_usize, validate_resource_sufficiency};
use super::scoring::{
  calculate_group_hw_component_summary, get_best_candidate_in_hsm,
  keep_iterating_final_hsm, parse_hw_pattern, resolve_hw_description_to_xnames,
};
use super::*;

// ---- parse_hw_pattern ----

#[test]
fn parse_hw_pattern_valid() {
  let input = vec!["a100", "4", "epyc", "10"];
  let (names, counts) = parse_hw_pattern(&input).unwrap();
  assert_eq!(names, vec!["a100", "epyc"]);
  assert_eq!(counts.get("a100"), Some(&4));
  assert_eq!(counts.get("epyc"), Some(&10));
}

#[test]
fn parse_hw_pattern_single_pair() {
  let input = vec!["instinct", "8"];
  let (names, counts) = parse_hw_pattern(&input).unwrap();
  assert_eq!(names, vec!["instinct"]);
  assert_eq!(counts.get("instinct"), Some(&8));
}

#[test]
fn parse_hw_pattern_empty() {
  let input: Vec<&str> = vec![];
  let (names, counts) = parse_hw_pattern(&input).unwrap();
  assert!(names.is_empty());
  assert!(counts.is_empty());
}

#[test]
fn parse_hw_pattern_odd_elements_errors() {
  let input = vec!["a100", "4", "epyc"];
  assert!(parse_hw_pattern(&input).is_err());
}

#[test]
fn parse_hw_pattern_non_numeric_count_errors() {
  let input = vec!["a100", "four"];
  assert!(parse_hw_pattern(&input).is_err());
}

#[test]
fn parse_hw_pattern_negative_count() {
  let input = vec!["a100", "-3"];
  let (_, counts) = parse_hw_pattern(&input).unwrap();
  assert_eq!(counts.get("a100"), Some(&-3));
}

#[test]
fn parse_hw_pattern_sorted_output() {
  let input = vec!["zebra", "1", "alpha", "2", "mid", "3"];
  let (names, _) = parse_hw_pattern(&input).unwrap();
  assert_eq!(names, vec!["alpha", "mid", "zebra"]);
}

// ---- calculate_group_hw_component_summary ----

#[test]
fn summary_empty_input() {
  let input: Vec<(String, HashMap<String, usize>)> = vec![];
  let result = calculate_group_hw_component_summary(&input);
  assert!(result.is_empty());
}

#[test]
fn summary_single_node() {
  let mut hw = HashMap::new();
  hw.insert("a100".to_string(), 4);
  hw.insert("epyc".to_string(), 2);
  let input = vec![("x1000c0s0b0n0".to_string(), hw)];
  let result = calculate_group_hw_component_summary(&input);
  assert_eq!(result.get("a100"), Some(&4));
  assert_eq!(result.get("epyc"), Some(&2));
}

#[test]
fn summary_multiple_nodes() {
  let mut hw1 = HashMap::new();
  hw1.insert("a100".to_string(), 4);
  hw1.insert("epyc".to_string(), 2);
  let mut hw2 = HashMap::new();
  hw2.insert("a100".to_string(), 2);
  hw2.insert("instinct".to_string(), 8);
  let input = vec![
    ("x1000c0s0b0n0".to_string(), hw1),
    ("x1000c0s1b0n0".to_string(), hw2),
  ];
  let result = calculate_group_hw_component_summary(&input);
  assert_eq!(result.get("a100"), Some(&6));
  assert_eq!(result.get("epyc"), Some(&2));
  assert_eq!(result.get("instinct"), Some(&8));
}

// ---- keep_iterating_final_hsm ----

#[test]
fn keep_iterating_when_current_exceeds_final() {
  let final_summary = HashMap::from([("a100".to_string(), 4)]);
  let current_summary = HashMap::from([("a100".to_string(), 6)]);
  assert!(keep_iterating_final_hsm(&final_summary, &current_summary));
}

#[test]
fn stop_iterating_when_current_equals_final() {
  let final_summary = HashMap::from([("a100".to_string(), 4)]);
  let current_summary = HashMap::from([("a100".to_string(), 4)]);
  assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
}

#[test]
fn stop_iterating_when_current_below_final() {
  let final_summary = HashMap::from([("a100".to_string(), 4)]);
  let current_summary = HashMap::from([("a100".to_string(), 2)]);
  assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
}

#[test]
fn stop_iterating_when_component_missing_from_current() {
  let final_summary = HashMap::from([("a100".to_string(), 4)]);
  let current_summary = HashMap::new();
  assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
}

#[test]
fn keep_iterating_mixed_components() {
  let final_summary =
    HashMap::from([("a100".to_string(), 4), ("epyc".to_string(), 10)]);
  let current_summary =
    HashMap::from([("a100".to_string(), 4), ("epyc".to_string(), 12)]);
  assert!(keep_iterating_final_hsm(&final_summary, &current_summary));
}

// ---- get_best_candidate_in_hsm ----

#[test]
fn best_candidate_empty_inputs() {
  let mut scores: Vec<(String, f64)> = vec![];
  let hw: Vec<(String, HashMap<String, usize>)> = vec![];
  assert!(get_best_candidate_in_hsm(&mut scores, &hw).is_none());
}

#[test]
fn best_candidate_highest_score_wins() {
  let mut scores = vec![
    ("x1000c0s0b0n0".to_string(), 2.0),
    ("x1000c0s1b0n0".to_string(), 5.0),
    ("x1000c0s2b0n0".to_string(), 3.0),
  ];
  let hw = vec![
    (
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    ),
    (
      "x1000c0s1b0n0".to_string(),
      HashMap::from([("a100".to_string(), 2)]),
    ),
    (
      "x1000c0s2b0n0".to_string(),
      HashMap::from([("a100".to_string(), 1)]),
    ),
  ];
  let result = get_best_candidate_in_hsm(&mut scores, &hw).unwrap();
  assert_eq!(result.0.0, "x1000c0s1b0n0");
  assert_eq!(result.0.1, 5.0);
  assert_eq!(result.1.get("a100"), Some(&2));
}

// ---- parse_hw_pattern_usize ----

#[test]
fn parse_hw_pattern_usize_valid() {
  let (names, counts) =
    parse_hw_pattern_usize("tasna", "a100:4:epyc:10").unwrap();
  assert_eq!(names, vec!["a100", "epyc"]);
  assert_eq!(counts.get("a100"), Some(&4));
  assert_eq!(counts.get("epyc"), Some(&10));
}

#[test]
fn parse_hw_pattern_usize_single_pair() {
  let (names, counts) = parse_hw_pattern_usize("group1", "instinct:8").unwrap();
  assert_eq!(names, vec!["instinct"]);
  assert_eq!(counts.get("instinct"), Some(&8));
}

#[test]
fn parse_hw_pattern_usize_odd_elements_errors() {
  assert!(parse_hw_pattern_usize("g", "a100:4:epyc").is_err());
}

#[test]
fn parse_hw_pattern_usize_non_numeric_count_errors() {
  assert!(parse_hw_pattern_usize("g", "a100:four").is_err());
}

#[test]
fn parse_hw_pattern_usize_negative_count_errors() {
  assert!(parse_hw_pattern_usize("g", "a100:-3").is_err());
}

#[test]
fn parse_hw_pattern_usize_sorted_output() {
  let (names, _) =
    parse_hw_pattern_usize("g", "zebra:1:alpha:2:mid:3").unwrap();
  assert_eq!(names, vec!["alpha", "mid", "zebra"]);
}

#[test]
fn parse_hw_pattern_usize_lowercased() {
  let (names, counts) = parse_hw_pattern_usize("GROUP", "A100:4").unwrap();
  assert_eq!(names, vec!["a100"]);
  assert_eq!(counts.get("a100"), Some(&4));
}

// ---- validate_resource_sufficiency ----

#[test]
fn validate_sufficiency_passes() {
  let target_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 4)]),
  )];
  let parent_hw = vec![(
    "x1000c0s1b0n0".to_string(),
    HashMap::from([("a100".to_string(), 8)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 10)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok()
  );
}

#[test]
fn validate_sufficiency_fails_insufficient() {
  let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
  let parent_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 2)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 10)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err()
  );
}

#[test]
fn validate_sufficiency_fails_missing_component() {
  let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
  let parent_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("epyc".to_string(), 10)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 1)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err()
  );
}

#[test]
fn validate_sufficiency_exact_match() {
  let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
  let parent_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 4)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 4)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok()
  );
}

#[test]
fn validate_sufficiency_combines_target_and_parent() {
  let target_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 3)]),
  )];
  let parent_hw = vec![(
    "x1000c0s1b0n0".to_string(),
    HashMap::from([("a100".to_string(), 3)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 6)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok()
  );
}

#[test]
fn validate_sufficiency_no_double_count_overlap() {
  let target_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 4)]),
  )];
  let parent_hw = vec![(
    "x1000c0s0b0n0".to_string(),
    HashMap::from([("a100".to_string(), 4)]),
  )];
  let requested = HashMap::from([("a100".to_string(), 5)]);
  assert!(
    validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err()
  );
}

// ---- resolve_hw_description_to_xnames (pin / unpin integration) ----

#[tokio::test]
pub async fn test_group_hw_management_pin_1() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let group_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 15),
      ]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("a100".to_string(), 4),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
        ("a100".to_string(), 4),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("Memory 16384".to_string(), 32),
        ("epyc".to_string(), 1),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
  ];

  let group_nodes_free_hw_conters = vec![
    (
      "x1000c1s7b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1000c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
  ];

  let (target_group_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      HwClusterMode::Pin,
      group_zinal_hw_counters,
      group_nodes_free_hw_conters,
      &user_request_hw_summary,
    )
    .unwrap();

  let target_group_hw_summary: HashMap<String, usize> =
    calculate_group_hw_component_summary(
      &target_group_node_hw_component_count_vec,
    );

  let mut success = true;
  for (hw_component, qty) in user_request_hw_summary {
    if !target_group_hw_summary.contains_key(&hw_component)
      || qty > *target_group_hw_summary.get(&hw_component).unwrap()
    {
      success = false;
    }
  }

  assert!(success);
}

#[tokio::test]
pub async fn test_group_hw_management_pin_2() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let group_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let group_nodes_free_hw_conters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let (target_group_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      HwClusterMode::Pin,
      group_zinal_hw_counters.clone(),
      group_nodes_free_hw_conters,
      &user_request_hw_summary,
    )
    .unwrap();

  let target_group_hw_summary: HashMap<String, usize> =
    calculate_group_hw_component_summary(
      &target_group_node_hw_component_count_vec,
    );

  let mut success = true;
  for (hw_component, qty) in &user_request_hw_summary {
    if !target_group_hw_summary.contains_key(hw_component)
      || *qty > *target_group_hw_summary.get(hw_component).unwrap()
    {
      success = false;
    }
  }

  // Pinning: new target should maximise nodes from old target
  success = success
    && target_group_node_hw_component_count_vec.iter().all(
      |(new_target_xname, _)| {
        group_zinal_hw_counters
          .iter()
          .any(|(old_target_xname, _)| old_target_xname == new_target_xname)
      },
    );

  assert!(success);
}

#[tokio::test]
pub async fn test_group_hw_management_unpin_1() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let group_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 15),
      ]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("a100".to_string(), 4),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
        ("a100".to_string(), 4),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("Memory 16384".to_string(), 32),
        ("epyc".to_string(), 1),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
  ];

  let group_nodes_free_hw_conters = vec![
    (
      "x1000c1s7b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1000c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
  ];

  let (target_group_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      HwClusterMode::Unpin,
      group_zinal_hw_counters,
      group_nodes_free_hw_conters,
      &user_request_hw_summary,
    )
    .unwrap();

  let group_hsm_hw_summary: HashMap<String, usize> =
    calculate_group_hw_component_summary(
      &target_group_node_hw_component_count_vec,
    );

  let mut success = true;
  for (hw_component, qty) in user_request_hw_summary {
    if !group_hsm_hw_summary.contains_key(&hw_component)
      || qty > *group_hsm_hw_summary.get(&hw_component).unwrap()
    {
      success = false;
    }
  }

  assert!(success);
}

#[tokio::test]
pub async fn test_hsm_hw_management_unpin_2() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let hsm_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let hsm_nodes_free_hw_conters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let (target_hsm_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      HwClusterMode::Unpin,
      hsm_zinal_hw_counters,
      hsm_nodes_free_hw_conters,
      &user_request_hw_summary,
    )
    .unwrap();

  let target_hsm_hw_summary: HashMap<String, usize> =
    calculate_group_hw_component_summary(
      &target_hsm_node_hw_component_count_vec,
    );

  let mut success = true;
  for (hw_component, qty) in user_request_hw_summary {
    if !target_hsm_hw_summary.contains_key(&hw_component)
      || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
    {
      success = false;
    }
  }

  assert!(success);
}
