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
  /// The `site = "..."` value from `cli.toml` itself, captured
  /// separately from `current_site` so the renderer can flag
  /// `--site` overrides. `None` when the key is absent or empty.
  pub configured_site: Option<String>,
  /// JWT-derived per-invocation facts. `None` when the command ran
  /// without a token — typically because no site is configured.
  pub session: Option<SessionSummary>,
  /// Active default group from `hsm_group = "..."`. `None` when the
  /// key is absent or empty.
  pub current_group: Option<String>,
}

/// Render the text-mode output to a `String`. Split out from `print`
/// so tests can assert against it without capturing stdout.
fn render_text(summary: &ConfigSummary) -> String {
  let mut out = String::new();
  push_local_section(&mut out, summary);
  out.push('\n');
  push_jwt_section(&mut out, summary.session.as_ref());
  out.push('\n');
  push_server_section(&mut out, summary.session.as_ref());
  out
}

fn push_local_section(out: &mut String, summary: &ConfigSummary) {
  out.push_str(&format!(
    "From local config file ({}):\n",
    summary.config_file
  ));
  let site_value = match (&summary.current_site, &summary.configured_site) {
    (Some(cur), Some(cfg)) if cur != cfg => {
      format!("{cur} (--site override; cli.toml: {cfg})")
    }
    (Some(cur), _) => cur.clone(),
    (None, _) => "(unset)".to_string(),
  };
  let group_value = summary
    .current_group
    .clone()
    .unwrap_or_else(|| "(unset)".to_string());
  let rows = [
    ("Log level", summary.log_level.as_str()),
    ("Current site", site_value.as_str()),
    ("Current group", group_value.as_str()),
  ];
  push_aligned_rows(out, &rows);
}

fn push_jwt_section(out: &mut String, session: Option<&SessionSummary>) {
  out.push_str("From JWT token:\n");
  match session {
    None => out.push_str("  (unavailable — no site selected)\n"),
    Some(s) => {
      let admin = if s.is_admin { "yes" } else { "no" };
      let read_only = if s.is_read_only { "yes" } else { "no" };
      let rows = [
        ("Username", s.username.as_str()),
        ("Name", s.name.as_str()),
        ("Admin", admin),
        ("Read-only", read_only),
      ];
      push_aligned_rows(out, &rows);
    }
  }
}

fn push_server_section(out: &mut String, session: Option<&SessionSummary>) {
  out.push_str("From server API:\n");
  match session {
    None => out.push_str("  (unavailable — no site selected)\n"),
    Some(s) => {
      let groups = if s.accessible_groups.is_empty() {
        "(none)".to_string()
      } else {
        s.accessible_groups.join(", ")
      };
      let rows = [("Accessible groups", groups.as_str())];
      push_aligned_rows(out, &rows);
    }
  }
}

/// Print rows as `  Label: value`, right-padding the colon-appended
/// labels to the longest in this slice so the colons line up. Padding
/// is per-section, not global — each block stays self-contained.
fn push_aligned_rows(out: &mut String, rows: &[(&str, &str)]) {
  // `+1` accounts for the colon we append to each label below.
  let width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0) + 1;
  for (k, v) in rows {
    let label = format!("{k}:");
    out.push_str(&format!("  {label:<width$} {v}\n"));
  }
}

/// Render `summary` to stdout. Plain text by default (three sections
/// grouped by provenance); structured JSON when `output_opt` is
/// `Some("json")`.
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
    print!("{}", render_text(summary));
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
      configured_site: Some("alps".to_string()),
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
  fn json_mode_emits_configured_site() {
    let s = sample();
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["configured_site"], "alps");

    let mut s2 = sample();
    s2.configured_site = None;
    let json2 = serde_json::to_string(&s2).unwrap();
    let v2: Value = serde_json::from_str(&json2).unwrap();
    assert!(v2["configured_site"].is_null());
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

  #[test]
  fn text_mode_annotates_site_override() {
    let mut s = sample();
    s.current_site = Some("alps".to_string());
    s.configured_site = Some("daint".to_string());
    let out = render_text(&s);
    assert!(
      out.contains("alps (--site override; cli.toml: daint)"),
      "expected override annotation, got:\n{out}"
    );
  }

  #[test]
  fn text_mode_no_override_when_sites_match() {
    let mut s = sample();
    s.current_site = Some("alps".to_string());
    s.configured_site = Some("alps".to_string());
    let out = render_text(&s);
    assert!(
      !out.contains("override"),
      "unexpected override marker:\n{out}"
    );
    assert!(
      out.contains("Current site:  alps"),
      "missing aligned site line:\n{out}"
    );
  }

  #[test]
  fn text_mode_renders_three_section_headers() {
    let out = render_text(&sample());
    assert!(
      out.contains("From local config file ("),
      "missing local section:\n{out}"
    );
    assert!(
      out.contains("From JWT token:"),
      "missing JWT section:\n{out}"
    );
    assert!(
      out.contains("From server API:"),
      "missing server section:\n{out}"
    );
  }

  #[test]
  fn text_mode_no_session_shows_unavailable_in_both_sections() {
    let mut s = sample();
    s.session = None;
    let out = render_text(&s);
    assert!(
      out.contains("From JWT token:"),
      "missing JWT section header:\n{out}"
    );
    assert!(
      out.contains("From server API:"),
      "missing server section header:\n{out}"
    );
    let unavailable_count =
      out.matches("(unavailable — no site selected)").count();
    assert_eq!(
      unavailable_count, 2,
      "expected unavailable line in both sections, got:\n{out}"
    );
  }
}
