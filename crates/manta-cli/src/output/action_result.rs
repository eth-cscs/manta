//! Renderer for the "side-effect" results that mutating commands
//! produce — `add`, `delete`, `update`, `apply`, `migrate`, `power`,
//! `config set/unset`, and friends.
//!
//! ## Subcommands
//!
//! Used by every mutating verb's success path, every `--dry-run`
//! preview path (via [`preview_request`]), and `config set` /
//! `config unset`.
//!
//! ## Output formats
//!
//! - text (default): a single short status line like "Group 'X'
//!   added" or "Boot parameters created successfully".
//! - JSON (`-o json`): `{"status":"ok","message":"..."}`, with an
//!   optional `"data": ...` field when the helper variant
//!   [`print_with_data`] is used.
//!
//! This lets shell-script callers branch on success without screen-
//! scraping the human text. For commands that need to surface
//! structured data alongside the status (e.g. an ID assigned by the
//! backend, a list of affected nodes), use [`print_with_data`] and
//! pass the payload as `serde_json::Value`.

use anyhow::{Context, Result};
use serde_json::json;

/// Format a single status message as the JSON envelope. Pure helper
/// extracted from [`print()`] so unit tests can assert the schema
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
  let value = json!({"status": "ok", "message": message, "data": data_value});
  serde_json::to_string(&value)
    .context("Failed to serialize action result to JSON")
}

/// Print a single status message. Plain text by default, JSON object
/// `{"status":"ok","message":"..."}` when `output_opt` is `Some("json")`.
///
/// # Errors
///
/// Returns `Err` if JSON serialisation fails (only reachable on the
/// JSON path).
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
///
/// # Errors
///
/// Returns `Err` if `data` cannot be converted to JSON or the
/// resulting envelope cannot be serialised.
pub fn print_with_data<T: serde::Serialize>(
  message: &str,
  data: &T,
  output_opt: Option<&str>,
) -> Result<()> {
  if let Some("json") = output_opt {
    println!("{}", json_envelope_with_data(message, data)?);
  } else {
    println!("{message}");
    println!(
      "{}",
      serde_json::to_string_pretty(data)
        .context("Failed to pretty-print data payload")?
    );
  }
  Ok(())
}

/// `--dry-run` preview for mutating verbs: render a
/// `Would <METHOD> <path>:` line followed by the request body that
/// would have been sent. Routes through [`print_with_data`] so dry-run
/// preview honours `-o json` just like the live success path. Use this
/// from every mutating dispatcher's dry-run branch so the format stays
/// uniform across verbs.
///
/// # Errors
///
/// Propagates serialisation failures from [`print_with_data`].
pub fn preview_request<T: serde::Serialize>(
  method: &str,
  path: &str,
  body: &T,
  output_opt: Option<&str>,
) -> Result<()> {
  print_with_data(&format!("Would {method} {path}:"), body, output_opt)
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
    let rendered = json_envelope_with_data("endpoint added", &payload).unwrap();
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
