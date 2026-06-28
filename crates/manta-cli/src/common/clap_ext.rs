//! Tiny extension trait for clap's `ArgMatches` to factor out the
//! `get_one::<String>(name).context(...)` boilerplate that every
//! handler used to repeat.
//!
//! The three methods cover the cases that account for ~95% of CLI
//! argument extraction:
//!
//! - [`ArgMatchesExt::req_str`] for required string arguments. Auto-
//!   generates the `"'<name>' argument is mandatory"` error message.
//! - [`ArgMatchesExt::opt_str`] for optional string arguments returned
//!   as `Option<&str>` (the borrowing form, which is what most
//!   downstream functions accept).
//! - [`ArgMatchesExt::opt_string`] for optional string arguments
//!   returned as `Option<String>` (when ownership is needed, e.g. to
//!   build a struct that outlives the matches).
//!
//! Typed-value extraction (`u8`, `PathBuf`, etc.) stays on the native
//! `ArgMatches::get_one::<T>` since each call site uses different
//! types; abstracting that one is not worth the generics.

use anyhow::{Context, Result};
use clap::ArgMatches;

/// Convenience accessors for the three string-extraction patterns the
/// CLI uses repeatedly. See the module docs for guidance.
pub trait ArgMatchesExt {
  /// Get a required `&str` argument. Returns an error with the message
  /// `"'<name>' argument is mandatory"` when missing, matching the
  /// convention every handler used to spell out by hand.
  ///
  /// # Errors
  ///
  /// Returns an `anyhow::Error` when `name` is absent from the
  /// parsed matches.
  fn req_str(&self, name: &'static str) -> Result<&str>;

  /// Get an optional `&str` argument, mirroring
  /// `get_one::<String>(name).map(String::as_str)`.
  fn opt_str(&self, name: &'static str) -> Option<&str>;

  /// Get an optional owned `String` argument, mirroring
  /// `get_one::<String>(name).cloned()`. Use when the value must
  /// outlive the matches (e.g. it's moved into a struct field).
  fn opt_string(&self, name: &'static str) -> Option<String>;

  /// Resolve the result-set size cap from `--most-recent` /
  /// `--limit`. When `--most-recent` is set it takes precedence and
  /// returns `Some(1)`; otherwise `--limit` (if set) is returned.
  fn limit_or_most_recent(&self) -> Option<u8>;
}

/// Resolve the target node expression: when `hsm_group` is `Some`, the
/// node expression is suppressed (the server expands the group to its
/// member xnames); otherwise the caller's `nodes` value is passed
/// through.
///
/// Encodes the shared rule used by both `add kernel-parameters` and
/// `delete kernel-parameters`: HSM-group targeting and direct xname
/// targeting are mutually exclusive, and the group wins when set.
pub fn resolve_node_target<'a>(
  hsm_group: Option<&'a str>,
  nodes: Option<&'a str>,
) -> Option<&'a str> {
  if hsm_group.is_none() { nodes } else { None }
}

impl ArgMatchesExt for ArgMatches {
  fn req_str(&self, name: &'static str) -> Result<&str> {
    self
      .get_one::<String>(name)
      .map(String::as_str)
      .with_context(|| format!("'{name}' argument is mandatory"))
  }

  fn opt_str(&self, name: &'static str) -> Option<&str> {
    self.get_one::<String>(name).map(String::as_str)
  }

  fn opt_string(&self, name: &'static str) -> Option<String> {
    self.get_one::<String>(name).cloned()
  }

  fn limit_or_most_recent(&self) -> Option<u8> {
    if let Some(true) = self.get_one("most-recent") {
      Some(1u8)
    } else {
      self.get_one::<u8>("limit").copied()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{Arg, Command};

  fn cmd() -> Command {
    Command::new("test")
      .arg(Arg::new("required").long("required"))
      .arg(Arg::new("optional").long("optional"))
  }

  #[test]
  fn req_str_returns_value_when_present() {
    let m = cmd().get_matches_from(["test", "--required", "abc"]);
    assert_eq!(m.req_str("required").unwrap(), "abc");
  }

  #[test]
  fn req_str_errors_when_missing_with_name_in_message() {
    let m = cmd().get_matches_from(["test"]);
    let err = m.req_str("required").unwrap_err().to_string();
    assert!(err.contains("'required'"), "got: {err}");
    assert!(err.contains("mandatory"), "got: {err}");
  }

  #[test]
  fn opt_str_returns_some_when_present() {
    let m = cmd().get_matches_from(["test", "--optional", "xyz"]);
    assert_eq!(m.opt_str("optional"), Some("xyz"));
  }

  #[test]
  fn opt_str_returns_none_when_missing() {
    let m = cmd().get_matches_from(["test"]);
    assert_eq!(m.opt_str("optional"), None);
  }

  #[test]
  fn opt_string_returns_owned_when_present() {
    let m = cmd().get_matches_from(["test", "--optional", "owned"]);
    assert_eq!(m.opt_string("optional"), Some("owned".to_string()));
  }
}
