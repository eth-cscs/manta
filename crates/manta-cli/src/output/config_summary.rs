//! Renderer for [`ConfigSummary`].
//!
//! Called by `manta config show`. Supported output formats:
//! **text** (default — multi-line human-readable block) and **JSON**
//! (`-o json` — a single structured object suitable for `jq`).
//!
//! The struct is built by [`crate::dispatch::config::show`] from a
//! mix of the parsed CLI config file and the per-invocation
//! [`crate::common::session::SessionContext`] attached to `AppContext`.

use anyhow::{Context, Result};
use serde::Serialize;

/// JWT-derived per-invocation facts surfaced by `manta config show`.
/// Mirrors [`crate::common::session::SessionContext`]; the renderer
/// keeps its own copy so the output module doesn't have to depend on
/// the AppContext type.
#[derive(Debug, Serialize)]
pub struct SessionSummary {
  pub username: String,
  pub name: String,
  pub is_admin: bool,
  pub is_read_only: bool,
  pub accessible_groups: Vec<String>,
}

/// All the values surfaced by `manta config show`.
#[derive(Debug, Serialize)]
pub struct ConfigSummary {
  /// Resolved path of the loaded `cli.toml`.
  pub config_file: String,
  /// `EnvFilter` directive string from `cli.toml`.
  pub log_level: String,
  /// The resolved active site for this invocation: the `--site`
  /// override when given, otherwise `site = "..."` from `cli.toml`.
  /// `None` when neither is set (`null` in JSON).
  pub current_site: Option<String>,
  /// JWT-derived per-invocation facts. `None` when the command ran
  /// without a token — typically because no site is configured.
  pub session: Option<SessionSummary>,
  /// Active default group from `hsm_group = "..."`. `None` when the
  /// key is absent or empty.
  pub current_group: Option<String>,
}

/// Render `summary` to stdout. Plain text by default (one line per
/// field); structured JSON when `output_opt` is `Some("json")`.
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
    match &summary.session {
      Some(s) => {
        println!("Username (from token): {}", s.username);
        println!("Name (from token): {}", s.name);
        println!(
          "Admin (from token): {}",
          if s.is_admin { "yes" } else { "no" }
        );
        println!(
          "Read-only (from token): {}",
          if s.is_read_only { "yes" } else { "no" }
        );
        let groups = if s.accessible_groups.is_empty() {
          "(none)".to_string()
        } else {
          s.accessible_groups.join(", ")
        };
        println!("Accessible groups (from token+server): {groups}");
      }
      None => {
        println!("Session: (no site selected, token not resolved)");
      }
    }
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
      session: Some(SessionSummary {
        username: "alice".to_string(),
        name: "Alice Smith".to_string(),
        is_admin: false,
        is_read_only: false,
        accessible_groups: vec!["compute".to_string(), "uan".to_string()],
      }),
      current_group: Some("compute".to_string()),
    }
  }

  #[test]
  fn text_mode_renders_without_panicking() {
    print(&sample(), None).unwrap();
    print(&sample(), Some("table")).unwrap();
  }

  #[test]
  fn json_mode_emits_session_subobject() {
    let s = sample();
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["config_file"], "/home/u/.config/manta/cli.toml");
    assert_eq!(v["log_level"], "info");
    assert_eq!(v["current_site"], "alps");
    assert_eq!(v["session"]["username"], "alice");
    assert_eq!(v["session"]["is_admin"], false);
    assert_eq!(v["session"]["is_read_only"], false);
    assert_eq!(v["session"]["accessible_groups"][0], "compute");
    assert_eq!(v["current_group"], "compute");
    // The old top-level `read_only` and `groups_available` keys are gone.
    assert!(v.get("read_only").is_none(), "no top-level read_only");
    assert!(
      v.get("groups_available").is_none(),
      "no top-level groups_available"
    );
  }

  #[test]
  fn session_none_serialises_as_null() {
    let mut s = sample();
    s.session = None;
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["session"].is_null());
  }

  #[test]
  fn current_site_none_renders_without_panic() {
    let mut s = sample();
    s.current_site = None;
    print(&s, None).unwrap();
  }
}
