//! CLI-local read-only mode: refuses backend-mutating verbs when the
//! `read_only = true` setting is active in `cli.toml`.
//!
//! Pure policy module — no I/O, no JWT parsing. The chokepoint in
//! `crate::dispatch::process::process_cli` is a single call to
//! [`read_only_gate`]; everything else here is reusable building
//! blocks. The toggle itself is wired by `manta config set/unset
//! read-only`.

use clap::ArgMatches;

/// Top-level CLI verbs that change backend state.
pub const MUTATING_VERBS: &[&str] =
  &["add", "apply", "delete", "migrate", "power", "run", "restore"];

/// Top-level CLI verbs that do not change backend state. Held
/// alongside [`MUTATING_VERBS`] so the consistency test can assert
/// `build_cli()`'s subcommand set equals their union.
pub const READ_ONLY_VERBS: &[&str] = &[
  "get",
  "console",
  "log",
  "backup",
  "config",
  "upgrade",
  "gen-autocomplete",
  "gen-man",
];

/// `true` when `--dry-run` is set on the leaf subcommand reachable
/// from `verb_matches`. Walks `subcommand()` to the deepest child
/// then checks the flag. `--dry-run` is only declared on mutating
/// verbs' leaf subcommands, so presence anywhere implies we are
/// under a mutating verb.
pub fn dry_run_set(verb_matches: &ArgMatches) -> bool {
  let mut current = verb_matches;
  while let Some((_, child)) = current.subcommand() {
    current = child;
  }
  current
    .try_get_one::<bool>("dry-run")
    .ok()
    .flatten()
    .copied()
    .unwrap_or(false)
}

/// Reject `verb` when `read_only` is `true`. Returns `Ok(())`
/// otherwise.
///
/// The error message names the verb and points the user at two
/// escape hatches: `--dry-run` (preview) and
/// `manta config unset read-only` (disable the policy).
pub fn ensure_can_mutate(
  read_only: bool,
  verb: &str,
) -> anyhow::Result<()> {
  if !read_only {
    return Ok(());
  }
  Err(anyhow::anyhow!(
    "manta is in read-only mode (`read_only = true` in cli.toml).\n\
     This `manta {verb} \u{2026}` invocation would change backend state and has been refused.\n\
     Re-run with `--dry-run` to preview, or disable the policy with `manta config unset read-only`."
  ))
}

/// Composed gate that the dispatch chokepoint calls. `Ok(())` if any
/// of: no subcommand parsed; `read_only` is `false`; the verb is not
/// in [`MUTATING_VERBS`]; or `--dry-run` is set. Otherwise returns
/// the [`ensure_can_mutate`] error.
pub fn read_only_gate(
  cli_root: &ArgMatches,
  read_only: bool,
) -> anyhow::Result<()> {
  let Some((verb, verb_matches)) = cli_root.subcommand() else {
    return Ok(());
  };
  if !read_only {
    return Ok(());
  }
  if !MUTATING_VERBS.contains(&verb) {
    return Ok(());
  }
  if dry_run_set(verb_matches) {
    return Ok(());
  }
  ensure_can_mutate(read_only, verb)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn matches(args: &[&str]) -> clap::ArgMatches {
    crate::build::build_cli().get_matches_from(args)
  }

  // ---- constants ----

  #[test]
  fn mutating_verbs_match_spec() {
    let expected = ["add", "apply", "delete", "migrate", "power", "run", "restore"];
    for v in expected {
      assert!(MUTATING_VERBS.contains(&v), "missing mutating verb: {v}");
    }
    assert_eq!(MUTATING_VERBS.len(), expected.len());
  }

  #[test]
  fn verb_lists_have_no_overlap() {
    for verb in MUTATING_VERBS {
      assert!(
        !READ_ONLY_VERBS.contains(verb),
        "verb {verb} is in both MUTATING_VERBS and READ_ONLY_VERBS"
      );
    }
  }

  #[test]
  fn every_top_level_verb_is_classified() {
    use std::collections::BTreeSet;
    let cli = crate::build::build_cli();
    let actual: BTreeSet<String> = cli
      .get_subcommands()
      .map(|s| s.get_name().to_string())
      .collect();
    let classified: BTreeSet<String> = MUTATING_VERBS
      .iter()
      .chain(READ_ONLY_VERBS.iter())
      .map(|s| s.to_string())
      .collect();
    let unclassified: Vec<_> = actual.difference(&classified).collect();
    let phantom: Vec<_> = classified.difference(&actual).collect();
    assert!(
      unclassified.is_empty(),
      "build_cli() declares verbs that are NOT in MUTATING_VERBS or READ_ONLY_VERBS: {unclassified:?}"
    );
    assert!(
      phantom.is_empty(),
      "MUTATING_VERBS / READ_ONLY_VERBS reference verbs build_cli() does not declare: {phantom:?}"
    );
  }

  // ---- dry_run_set ----

  #[test]
  fn dry_run_set_apply_boot_nodes_with_flag() {
    let m = matches(&[
      "manta", "apply", "boot", "nodes",
      "--boot-image", "abc",
      "--dry-run",
      "x1000c0s0b0n0",
    ]);
    let (_, verb_matches) = m.subcommand().unwrap();
    assert!(dry_run_set(verb_matches));
  }

  #[test]
  fn dry_run_set_apply_boot_nodes_without_flag() {
    let m = matches(&[
      "manta", "apply", "boot", "nodes",
      "--boot-image", "abc",
      "x1000c0s0b0n0",
    ]);
    let (_, verb_matches) = m.subcommand().unwrap();
    assert!(!dry_run_set(verb_matches));
  }

  #[test]
  fn dry_run_set_delete_session_with_flag() {
    let m = matches(&["manta", "delete", "session", "my-session", "--dry-run"]);
    let (_, verb_matches) = m.subcommand().unwrap();
    assert!(dry_run_set(verb_matches));
  }

  #[test]
  fn dry_run_set_get_sessions_returns_false() {
    let m = matches(&["manta", "get", "sessions"]);
    let (_, verb_matches) = m.subcommand().unwrap();
    assert!(!dry_run_set(verb_matches));
  }

  // ---- ensure_can_mutate ----

  #[test]
  fn ensure_can_mutate_allows_when_not_read_only() {
    assert!(ensure_can_mutate(false, "apply").is_ok());
  }

  #[test]
  fn ensure_can_mutate_blocks_when_read_only() {
    let err = ensure_can_mutate(true, "apply").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("apply"), "msg should name the verb: {msg}");
    assert!(msg.contains("--dry-run"), "msg should mention --dry-run: {msg}");
    assert!(
      msg.contains("manta config unset read-only"),
      "msg should point at the unset command: {msg}"
    );
  }

  // ---- read_only_gate ----

  #[test]
  fn gate_allows_when_read_only_off() {
    let m = matches(&[
      "manta", "apply", "boot", "nodes",
      "--boot-image", "abc",
      "x1000c0s0b0n0",
    ]);
    assert!(read_only_gate(&m, false).is_ok());
  }

  #[test]
  fn gate_blocks_mutating_verb_when_read_only_on() {
    let m = matches(&[
      "manta", "apply", "boot", "nodes",
      "--boot-image", "abc",
      "x1000c0s0b0n0",
    ]);
    let err = read_only_gate(&m, true).unwrap_err();
    assert!(format!("{err}").contains("apply"));
  }

  #[test]
  fn gate_allows_mutating_verb_with_dry_run_when_read_only_on() {
    let m = matches(&[
      "manta", "apply", "boot", "nodes",
      "--boot-image", "abc",
      "--dry-run",
      "x1000c0s0b0n0",
    ]);
    assert!(read_only_gate(&m, true).is_ok());
  }

  #[test]
  fn gate_allows_read_only_verb_when_read_only_on() {
    let m = matches(&["manta", "get", "sessions"]);
    assert!(read_only_gate(&m, true).is_ok());
  }
}
