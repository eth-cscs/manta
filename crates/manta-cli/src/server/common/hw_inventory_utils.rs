use serde_json::Value;

/// Extract memory DIMM capacities (MiB) from a node's
/// hardware inventory JSON.
pub fn get_list_memory_capacity_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<u64>> {
  hw_inventory
    .pointer("/Nodes/0/Memory")
    .and_then(Value::as_array)
    .map(|memory_list| {
      memory_list
        .iter()
        .map(|memory| {
          memory
            .pointer("/PopulatedFRU/MemoryFRUInfo/CapacityMiB")
            .unwrap_or(&serde_json::json!(0))
            .as_u64()
            .unwrap_or(0)
        })
        .collect::<Vec<u64>>()
    })
}

/// Extract processor model names from a node's hardware
/// inventory JSON.
pub fn get_list_processor_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/Processors")
    .and_then(Value::as_array)
    .map(|processor_list| {
      processor_list
        .iter()
        .filter_map(|processor| {
          processor
            .pointer("/PopulatedFRU/ProcessorFRUInfo/Model")
            .and_then(Value::as_str)
            .map(|s| s.to_string())
        })
        .collect::<Vec<String>>()
    })
}

/// Extract accelerator (GPU) model names from a node's
/// hardware inventory JSON.
pub fn get_list_accelerator_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/NodeAccels")
    .and_then(Value::as_array)
    .map(|accelerator_list| {
      accelerator_list
        .iter()
        .filter_map(|accelerator| {
          accelerator
            .pointer("/PopulatedFRU/NodeAccelFRUInfo/Model")
            .and_then(Value::as_str)
            .map(|s| s.to_string())
        })
        .collect::<Vec<String>>()
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  // ── get_list_memory_capacity_from_hw_inventory_value ──

  #[test]
  fn memory_capacity_returns_values_for_valid_json() {
    let hw = json!({
      "Nodes": [{
        "Memory": [
          {"PopulatedFRU": {"MemoryFRUInfo": {"CapacityMiB": 16384}}},
          {"PopulatedFRU": {"MemoryFRUInfo": {"CapacityMiB": 32768}}}
        ]
      }]
    });
    let result = get_list_memory_capacity_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec![16384, 32768]);
  }

  #[test]
  fn memory_capacity_returns_none_when_no_nodes() {
    let hw = json!({});
    assert!(get_list_memory_capacity_from_hw_inventory_value(&hw).is_none());
  }

  #[test]
  fn memory_capacity_returns_none_when_no_memory_key() {
    let hw = json!({"Nodes": [{"Processors": []}]});
    assert!(get_list_memory_capacity_from_hw_inventory_value(&hw).is_none());
  }

  #[test]
  fn memory_capacity_returns_empty_vec_for_empty_memory_array() {
    let hw = json!({"Nodes": [{"Memory": []}]});
    let result = get_list_memory_capacity_from_hw_inventory_value(&hw).unwrap();
    assert!(result.is_empty());
  }

  #[test]
  fn memory_capacity_defaults_to_zero_when_field_missing() {
    let hw = json!({
      "Nodes": [{
        "Memory": [
          {"PopulatedFRU": {"MemoryFRUInfo": {}}},
          {"PopulatedFRU": {"MemoryFRUInfo": {"CapacityMiB": 8192}}}
        ]
      }]
    });
    let result = get_list_memory_capacity_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec![0, 8192]);
  }

  // ── get_list_processor_model_from_hw_inventory_value ──

  #[test]
  fn processor_model_returns_values_for_valid_json() {
    let hw = json!({
      "Nodes": [{
        "Processors": [
          {"PopulatedFRU": {"ProcessorFRUInfo": {"Model": "AMD EPYC 7742"}}},
          {"PopulatedFRU": {"ProcessorFRUInfo": {"Model": "Intel Xeon Gold 6248"}}}
        ]
      }]
    });
    let result = get_list_processor_model_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec!["AMD EPYC 7742", "Intel Xeon Gold 6248"]);
  }

  #[test]
  fn processor_model_returns_none_when_no_nodes() {
    let hw = json!({});
    assert!(get_list_processor_model_from_hw_inventory_value(&hw).is_none());
  }

  #[test]
  fn processor_model_skips_entries_without_model_field() {
    let hw = json!({
      "Nodes": [{
        "Processors": [
          {"PopulatedFRU": {"ProcessorFRUInfo": {"Model": "AMD EPYC 7742"}}},
          {"PopulatedFRU": {"ProcessorFRUInfo": {}}},
          {"PopulatedFRU": {}}
        ]
      }]
    });
    let result = get_list_processor_model_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec!["AMD EPYC 7742"]);
  }

  #[test]
  fn processor_model_returns_empty_vec_for_empty_array() {
    let hw = json!({"Nodes": [{"Processors": []}]});
    let result = get_list_processor_model_from_hw_inventory_value(&hw).unwrap();
    assert!(result.is_empty());
  }

  // ── get_list_accelerator_model_from_hw_inventory_value ──

  #[test]
  fn accelerator_model_returns_values_for_valid_json() {
    let hw = json!({
      "Nodes": [{
        "NodeAccels": [
          {"PopulatedFRU": {"NodeAccelFRUInfo": {"Model": "NVIDIA A100"}}},
          {"PopulatedFRU": {"NodeAccelFRUInfo": {"Model": "NVIDIA H100"}}}
        ]
      }]
    });
    let result =
      get_list_accelerator_model_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec!["NVIDIA A100", "NVIDIA H100"]);
  }

  #[test]
  fn accelerator_model_returns_none_when_no_nodes() {
    let hw = json!({});
    assert!(get_list_accelerator_model_from_hw_inventory_value(&hw).is_none());
  }

  #[test]
  fn accelerator_model_skips_entries_without_model_field() {
    let hw = json!({
      "Nodes": [{
        "NodeAccels": [
          {"PopulatedFRU": {"NodeAccelFRUInfo": {"Model": "NVIDIA A100"}}},
          {"PopulatedFRU": {"NodeAccelFRUInfo": {}}},
          {"SomethingElse": {}}
        ]
      }]
    });
    let result =
      get_list_accelerator_model_from_hw_inventory_value(&hw).unwrap();
    assert_eq!(result, vec!["NVIDIA A100"]);
  }

  #[test]
  fn accelerator_model_returns_empty_vec_for_empty_array() {
    let hw = json!({"Nodes": [{"NodeAccels": []}]});
    let result =
      get_list_accelerator_model_from_hw_inventory_value(&hw).unwrap();
    assert!(result.is_empty());
  }
}
