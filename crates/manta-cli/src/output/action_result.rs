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

/// Format a single status message as the JSON envelope. Pure helper
/// extracted from [`print`] so unit tests can assert the schema
/// without capturing stdout.
fn json_envelope(message: &str) -> Result<String> {
  let value = json!({"status": "ok", "message": message});
  serde_json::to_string(&value)
    .context("Failed to serialize action result to JSON")
}

/// Format a status message + structured payload as the JSON envelope.
/// Pure helper extracted from [`print_with_data`].
fn json_envelope_with_data<T: serde::Serialize>(
  message: &str,
  data: &T,
) -> Result<String> {
  let data_value = serde_json::to_value(data)
    .context("Failed to convert data payload to JSON")?;
  let value =
    json!({"status": "ok", "message": message, "data": data_value});
  serde_json::to_string(&value)
    .context("Failed to serialize action result to JSON")
}

/// Print a single status message. Plain text by default, JSON object
/// `{"status":"ok","message":"..."}` when `output_opt` is `Some("json")`.
pub fn print(message: &str, output_opt: Option<&str>) -> Result<()> {
  match output_opt {
    Some("json") => println!("{}", json_envelope(message)?),
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
      println!("{}", json_envelope_with_data(message, data)?);
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

  #[test]
  fn json_envelope_has_status_and_message() {
    // Round-trip the rendered string so we're asserting against the
    // actual output of `json_envelope`, not a hand-built `json!{...}`
    // we hope matches it.
    let rendered = json_envelope("Group 'foo' added").unwrap();
    let parsed: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["message"], "Group 'foo' added");
  }

  #[test]
  fn json_envelope_with_data_attaches_payload() {
    let payload = json!({"id": "x3000c0s1b0", "enabled": true});
    let rendered =
      json_envelope_with_data("endpoint added", &payload).unwrap();
    let parsed: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["message"], "endpoint added");
    assert_eq!(parsed["data"]["id"], "x3000c0s1b0");
    assert_eq!(parsed["data"]["enabled"], true);
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
