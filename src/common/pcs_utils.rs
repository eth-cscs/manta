use comfy_table::Table;
use serde_json::Value;

pub fn print_summary_table(transition: Value, output: &str) {
  if output == "table" {
    let tasks: Vec<Value> = transition
      .get("tasks")
      .and_then(Value::as_array)
      .unwrap()
      .to_vec();

    let mut table = Table::new();

    println!(
      "\nTransition ID: {}",
      transition
        .get("transitionID")
        .and_then(Value::as_str)
        .unwrap()
    );
    println!(
      "Transition Status: {}",
      transition
        .get("transitionStatus")
        .and_then(Value::as_str)
        .unwrap()
    );

    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .set_header(vec!["XNAME", "Status", "Description"]);

    for task in tasks {
      table.add_row(vec![
        task.get("xname").and_then(Value::as_str).unwrap(),
        task.get("taskStatus").and_then(Value::as_str).unwrap(),
        task
          .get("taskStatusDescription")
          .and_then(Value::as_str)
          .unwrap(),
      ]);
    }

    println!("{table}");
  } else if output.to_lowercase() == "json" {
    println!("{}", serde_json::to_string_pretty(&transition).unwrap());
  }
}
