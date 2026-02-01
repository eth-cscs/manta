use comfy_table::Table;
use manta_backend_dispatcher::types::pcs::transitions::types::TransitionResponse;

pub fn print_summary_table(transition: TransitionResponse, output: &str) {
  if output == "table" {
    let mut table = Table::new();

    println!("\nTransition ID: {}", transition.transition_id);
    println!("Transition Status: {}", transition.transition_status);

    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .set_header(vec!["XNAME", "Status", "Description"]);

    for task in transition.tasks.iter() {
      table.add_row(vec![
        &task.xname,
        &task.task_status,
        &task.task_status_description,
      ]);
    }

    println!("{table}");
  } else if output.to_lowercase() == "json" {
    println!("{}", serde_json::to_string_pretty(&transition).unwrap());
  }
}
