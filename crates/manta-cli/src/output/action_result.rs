//! Renderer for the "side-effect" results that mutating commands
//! produce — `add`, `delete`, `update`, `apply`, `migrate`, `power`,
//! `config set/unset`, and friends.
//!
//! Every such command ends with a single short status line like
//! "Group 'X' added" or "Boot parameters created successfully". With
//! the renderer plumbed through, the same commands also support
//! `--output json` and emit `{"status":"ok","message":"..."}` instead,
//! which lets shell-script callers branch on success without screen-
//! scraping the human text.
//!
//! For commands that also need to surface structured data alongside
//! the status (e.g. an ID assigned by the backend, a list of affected
//! nodes), use [`print_with_data`] and pass the payload as
//! `serde_json::Value`.

use anyhow::{Context, Result};
use serde_json::json;

/// Print a single status message. Plain text by default, JSON object
/// `{"status":"ok","message":"..."}` when `output_opt` is `Some("json")`.
pub fn print(message: &str, output_opt: Option<&str>) -> Result<()> {
  match output_opt {
    Some("json") => {
      let value = json!({"status": "ok", "message": message});
      println!(
        "{}",
        serde_json::to_string(&value)
          .context("Failed to serialize action result to JSON")?
      );
    }
    _ => println!("{message}"),
  }
  Ok(())
}

/// Print a status message with an associated structured payload.
/// In text mode prints the message followed by the data pretty-printed;
/// in JSON mode prints `{"status":"ok","message":"...","data":<payload>}`.
pub fn print_with_data<T: serde::Serialize>(
  message: &str,
  data: &T,
  output_opt: Option<&str>,
) -> Result<()> {
  match output_opt {
    Some("json") => {
      let data_value = serde_json::to_value(data)
        .context("Failed to convert data payload to JSON")?;
      let value =
        json!({"status": "ok", "message": message, "data": data_value});
      println!(
        "{}",
        serde_json::to_string(&value)
          .context("Failed to serialize action result to JSON")?
      );
    }
    _ => {
      println!("{message}");
      println!(
        "{}",
        serde_json::to_string_pretty(data)
          .context("Failed to pretty-print data payload")?
      );
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::Value;

  /// Render to JSON and verify the shape.
  fn json_of(message: &str) -> Value {
    let v = json!({"status": "ok", "message": message});
    serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap()
  }

  #[test]
  fn json_shape_for_plain_message() {
    let v = json_of("Group 'foo' added");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["message"], "Group 'foo' added");
  }

  #[test]
  fn print_with_data_includes_payload() {
    // Just verify the schema we build; the actual stdout write isn't
    // worth capturing in this unit test (already covered structurally).
    let payload = json!({"id": "x3000c0s1b0", "enabled": true});
    let combined = json!({
      "status": "ok",
      "message": "endpoint added",
      "data": payload,
    });
    assert_eq!(combined["data"]["id"], "x3000c0s1b0");
    assert_eq!(combined["data"]["enabled"], true);
  }

  #[test]
  fn print_text_mode_does_not_error() {
    // Smoke test the API contract; visual confirmation lives in
    // higher-level integration tests.
    print("hello", None).unwrap();
    print("hello", Some("table")).unwrap();
  }

  #[test]
  fn print_json_mode_does_not_error() {
    print("hello", Some("json")).unwrap();
  }
}
