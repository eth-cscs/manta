use std::collections::HashMap;

use manta_backend_dispatcher::types::bss::BootParameters;

/// Group boot parameters by their (optionally filtered) kernel
/// parameter sets.
///
/// Returns a map from sorted kernel-param vectors to the list
/// of host xnames that share that exact parameter set.
pub fn group_boot_params_by_kernel_params(
  boot_parameters_vec: &[BootParameters],
  kernel_params_key_to_filter_opt: Option<&str>,
) -> HashMap<Vec<String>, Vec<String>> {
  let mut kernel_params_key_vec: Vec<String> =
    if let Some(highlight) = kernel_params_key_to_filter_opt {
      highlight
        .split(',')
        .map(|value| value.trim().to_string())
        .collect()
    } else {
      vec![]
    };

  kernel_params_key_vec.sort();

  let mut kernel_param_node_map: HashMap<Vec<String>, Vec<String>> =
    HashMap::new();

  for boot_parameters in boot_parameters_vec {
    let mut host_vec = boot_parameters.hosts.clone();
    let kernel_params = boot_parameters.params.clone();

    let kernel_params_vec: Vec<String> = kernel_params
      .split_whitespace()
      .map(str::to_string)
      .collect();

    let mut kernel_params_vec: Vec<String> = if !kernel_params_key_vec
      .is_empty()
    {
      kernel_params_vec
        .into_iter()
        .filter(|kp| kernel_params_key_vec.iter().any(|kp_k| kp.contains(kp_k)))
        .collect()
    } else {
      kernel_params_vec.clone()
    };

    kernel_params_vec.sort();

    kernel_param_node_map
      .entry(kernel_params_vec)
      .and_modify(|xname_vec| xname_vec.append(&mut host_vec))
      .or_insert(host_vec);
  }

  kernel_param_node_map
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_boot_params(hosts: Vec<&str>, params: &str) -> BootParameters {
    BootParameters {
      hosts: hosts.into_iter().map(String::from).collect(),
      params: params.to_string(),
      ..Default::default()
    }
  }

  #[test]
  fn group_by_identical_params() {
    let bp = vec![
      make_boot_params(vec!["x1000c0s0b0n0"], "ip=dhcp quiet"),
      make_boot_params(vec!["x1000c0s0b0n1"], "ip=dhcp quiet"),
    ];
    let result = group_boot_params_by_kernel_params(&bp, None);
    assert_eq!(result.len(), 1);
    let hosts = result.values().next().unwrap();
    assert_eq!(hosts.len(), 2);
  }

  #[test]
  fn group_by_different_params() {
    let bp = vec![
      make_boot_params(vec!["x1000c0s0b0n0"], "ip=dhcp quiet"),
      make_boot_params(vec!["x1000c0s0b0n1"], "ip=dhcp verbose"),
    ];
    let result = group_boot_params_by_kernel_params(&bp, None);
    assert_eq!(result.len(), 2);
  }

  #[test]
  fn filter_by_key() {
    let bp = vec![
      make_boot_params(vec!["x1000c0s0b0n0"], "ip=dhcp quiet console=tty0"),
      make_boot_params(vec!["x1000c0s0b0n1"], "ip=static quiet console=tty1"),
    ];
    let result = group_boot_params_by_kernel_params(&bp, Some("ip"));
    // Both have different ip= values so 2 groups
    assert_eq!(result.len(), 2);
    // Each group should only have the ip= param
    for (params, _) in &result {
      assert!(params.iter().all(|p| p.contains("ip")));
    }
  }

  #[test]
  fn filter_multiple_keys() {
    let bp = vec![make_boot_params(
      vec!["x1000c0s0b0n0"],
      "ip=dhcp quiet console=tty0 root=/dev/sda",
    )];
    let result = group_boot_params_by_kernel_params(&bp, Some("ip,console"));
    let params = result.keys().next().unwrap();
    assert_eq!(params.len(), 2);
    assert!(params.iter().any(|p| p.contains("ip")));
    assert!(params.iter().any(|p| p.contains("console")));
  }

  #[test]
  fn empty_input() {
    let result = group_boot_params_by_kernel_params(&[], None);
    assert!(result.is_empty());
  }

  #[test]
  fn filter_no_match_returns_empty_params_group() {
    let bp = vec![make_boot_params(vec!["x1000c0s0b0n0"], "ip=dhcp quiet")];
    let result = group_boot_params_by_kernel_params(&bp, Some("nonexistent"));
    // One group with empty params vec
    assert_eq!(result.len(), 1);
    let params = result.keys().next().unwrap();
    assert!(params.is_empty());
  }

  #[test]
  fn params_are_sorted_for_grouping() {
    // Same params in different order should group together
    let bp = vec![
      make_boot_params(vec!["x1000c0s0b0n0"], "b=2 a=1"),
      make_boot_params(vec!["x1000c0s0b0n1"], "a=1 b=2"),
    ];
    let result = group_boot_params_by_kernel_params(&bp, None);
    assert_eq!(
      result.len(),
      1,
      "Same params in different order should be grouped"
    );
  }
}
