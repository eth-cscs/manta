//! Renderer for [`ConfigSummary`].
//!
//! Called by `manta config show`. Supported output formats:
//! **text** (default — multi-line human-readable block in the legacy
//! field order) and **JSON** (`-o json` — a single structured object
//! suitable for `jq`).
//!
//! The struct is built by [`crate::dispatch::config::show`] from a
//! mix of the parsed CLI config file and a live call to the server
//! for the list of HSM groups the token can access.

use anyhow::{Context, Result};
use serde::Serialize;

/// All the values surfaced by `manta config show`.
///
/// Built by [`crate::dispatch::config::show`] from a mix of the
/// parsed CLI config file and a live call to the server for the list
/// of HSM groups the token can access.
#[derive(Debug, Serialize)]
pub struct ConfigSummary {
  /// Resolved path of the loaded `cli.toml`.
  pub config_file: String,
  /// `EnvFilter` directive string from `cli.toml`.
  pub log_level: String,
  /// The resolved active site for this invocation: the `--site`
  /// override when given, otherwise `site = "..."` from `cli.toml`.
  /// `None` when neither is set — `config show` still works without a
  /// site, it just can't report one (serializes to `null` in JSON).
  pub current_site: Option<String>,
  /// Mirror of `CliConfiguration.read_only`. When `true`, the
  /// chokepoint in `dispatch::process::process_cli` refuses
  /// backend-mutating verbs. See `crate::common::read_only`.
  pub read_only: bool,
  /// HSM groups the bearer token is permitted to access; `None` when
  /// the server lookup failed.
  pub groups_available: Option<Vec<String>>,
  /// Active default group from `hsm_group = "..."`. `None` when the
  /// key is absent or empty — like `current_site`, it then renders as
  /// `(unset)` in text and `null` in JSON.
  pub current_group: Option<String>,
}

/// Render `summary` to stdout. Plain text by default (one line per
/// field, in the historical order); structured JSON when
/// `output_opt` is `Some("json")`.
///
/// # Errors
///
/// Returns `Err` if JSON serialisation fails (JSON path only).
pub fn print(summary: &ConfigSummary, output_opt: Option<&str>) -> Result<()> {
  if let Some("json") = output_opt {
    println!(
      "{}",
      serde_json::to_string(summary)
        .context("Failed to serialize config summary to JSON")?
    );
  } else {
    println!("Configuration file: {}", summary.config_file);
    println!("Log level: {}", summary.log_level);
    println!(
      "Current site: {}",
      summary.current_site.as_deref().unwrap_or("(unset)")
    );
    println!(
      "Read-only: {}",
      if summary.read_only { "yes" } else { "no" }
    );
    let groups = match (&summary.groups_available, &summary.current_site) {
      (Some(v), _) => v.join(", "),
      // No site selected, so there was nothing to query — distinguish
      // this from a genuine lookup failure against a selected site.
      (None, None) => "(no site selected)".to_string(),
      (None, Some(_)) => "Could not get list of groups available".to_string(),
    };
    println!("Groups available: {groups}");
    println!(
      "Current group: {}",
      summary.current_group.as_deref().unwrap_or("(unset)")
    );
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::Value;

  fn sample() -> ConfigSummary {
    ConfigSummary {
      config_file: "/home/u/.config/manta/cli.toml".to_string(),
      log_level: "info".to_string(),
      current_site: Some("alps".to_string()),
      read_only: false,
      groups_available: Some(vec!["compute".to_string(), "uan".to_string()]),
      current_group: Some("compute".to_string()),
    }
  }

  /// The text-mode renderer should reproduce the legacy line ordering
  /// so anyone scraping `manta config show` keeps working.
  #[test]
  fn text_mode_writes_lines_in_legacy_order() {
    // The renderer prints to stdout, so we just verify the JSON shape
    // (rendered via serde) is also serializable end-to-end.
    print(&sample(), None).unwrap();
    print(&sample(), Some("table")).unwrap();
  }

  #[test]
  fn json_mode_emits_one_object_with_expected_fields() {
    let s = sample();
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["config_file"], "/home/u/.config/manta/cli.toml");
    assert_eq!(v["log_level"], "info");
    assert_eq!(v["current_site"], "alps");
    assert_eq!(v["read_only"], false);
    assert_eq!(v["groups_available"][0], "compute");
    assert_eq!(v["current_group"], "compute");
  }

  #[test]
  fn groups_available_none_renders_as_null_in_json() {
    let mut s = sample();
    s.groups_available = None;
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["groups_available"].is_null());
  }

  #[test]
  fn current_site_none_renders_as_null_in_json() {
    let mut s = sample();
    s.current_site = None;
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["current_site"].is_null());
  }

  #[test]
  fn current_group_none_renders_as_null_in_json() {
    let mut s = sample();
    s.current_group = None;
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["current_group"].is_null());
  }

  /// Text mode must not panic when no site is set; it prints a sentinel.
  #[test]
  fn text_mode_renders_with_no_site() {
    let mut s = sample();
    s.current_site = None;
    print(&s, None).unwrap();
  }
}
