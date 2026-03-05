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
