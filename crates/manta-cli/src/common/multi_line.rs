//! Multi-line wrapping helper for wide table cells.
//!
//! Used by `crate::output::node` and `crate::output::template` to
//! keep long comma-separated lists (xnames, nids, boot-set members)
//! readable in `comfy_table` output.

/// Render a list of node identifiers (xnames, NIDs, group members…)
/// as a comma-separated string, wrapping after every `num_columns`
/// entries with a newline.
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
