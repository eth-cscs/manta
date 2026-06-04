//! CLI-only display helpers (string formatting for tables, etc.).
//!
//! Anything here is consumed by `crate::output::*` table renderers
//! and has no business logic. The full xname/nid resolution lives in
//! `crate::service::node_ops` (server-side).

/// Render a list of node identifiers (xnames, NIDs, group members…)
/// as a comma-separated string, wrapping after every `num_columns`
/// entries with a newline. Used by the table renderers in
/// `cli::output` to keep wide columns readable.
pub fn string_vec_to_multi_line_string(
  nodes: Option<&[String]>,
  num_columns: usize,
) -> String {
  if num_columns == 0 {
    return String::new();
  }

  let mut members: String;

  match nodes {
    Some(nodes) if !nodes.is_empty() => {
      // Safe: guarded by !is_empty()
      members = nodes[0].to_string();

      for (i, node) in nodes.iter().enumerate().skip(1) {
        // iterate for the rest of the list
        if i % num_columns == 0 {
          // breaking the cell content into multiple lines (only 2 xnames per line)
          members.push_str(",\n");
        } else {
          members.push(',');
        }

        members.push_str(node);
      }
    }
    _ => members = String::new(),
  }

  members
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multi_line_none() {
    assert_eq!(string_vec_to_multi_line_string(None, 1), "");
  }

  #[test]
  fn multi_line_empty() {
    let nodes: Vec<String> = vec![];
    assert_eq!(string_vec_to_multi_line_string(Some(&nodes), 1), "");
  }

  #[test]
  fn multi_line_single_element() {
    let nodes = vec!["x1000c0s0b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 1),
      "x1000c0s0b0n0"
    );
  }

  #[test]
  fn multi_line_two_elements_one_column() {
    let nodes = vec!["x1000c0s0b0n0".to_string(), "x1000c0s1b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 1),
      "x1000c0s0b0n0,\nx1000c0s1b0n0"
    );
  }

  #[test]
  fn multi_line_two_elements_two_columns() {
    let nodes = vec!["x1000c0s0b0n0".to_string(), "x1000c0s1b0n0".to_string()];
    assert_eq!(
      string_vec_to_multi_line_string(Some(&nodes), 2),
      "x1000c0s0b0n0,x1000c0s1b0n0"
    );
  }

  #[test]
  fn multi_line_three_elements_two_columns() {
    let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    // i=1: 1%2=1 -> comma, i=2: 2%2=0 -> newline
    assert_eq!(string_vec_to_multi_line_string(Some(&nodes), 2), "a,b,\nc");
  }
}
