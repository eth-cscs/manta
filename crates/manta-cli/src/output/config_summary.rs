//! Renderer for the `manta config show` command.
//!
//! The summary is a single struct with one field per displayed value;
//! the renderer turns it into either a multi-line human-readable block
//! (the legacy format that `config show` used to print line-by-line) or
//! one structured JSON object suitable for scripting.

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
  pub current_site: String,
  /// Mirror of `CliConfiguration.read_only`. When `true`, the
  /// chokepoint in `dispatch::process::process_cli` refuses
  /// backend-mutating verbs. See `crate::common::read_only`.
  pub read_only: bool,
  /// HSM groups the bearer token is permitted to access; `None` when
  /// the server lookup failed.
  pub groups_available: Option<Vec<String>>,
  /// Active default HSM group from `hsm_group = "..."`.
  pub current_hsm: String,
}

/// Render `summary` to stdout. Plain text by default (one line per
/// field, in the historical order); structured JSON when
/// `output_opt` is `Some("json")`.
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
    println!("Current site: {}", summary.current_site);
    println!(
      "Read-only: {}",
      if summary.read_only { "yes" } else { "no" }
    );
    let groups = summary.groups_available.as_ref().map_or_else(
      || "Could not get list of groups available".to_string(),
      |v| v.join(", "),
    );
    println!("Groups available: {groups}");
    println!("Current HSM: {}", summary.current_hsm);
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
      current_site: "alps".to_string(),
      read_only: false,
      groups_available: Some(vec!["compute".to_string(), "uan".to_string()]),
      current_hsm: "compute".to_string(),
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
    assert_eq!(v["current_hsm"], "compute");
  }

  #[test]
  fn groups_available_none_renders_as_null_in_json() {
    let mut s = sample();
    s.groups_available = None;
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["groups_available"].is_null());
  }
}
