//! Clap command tree definition for the manta CLI.
//!
//! Top-level `build_cli()` lives here. Each top-level verb has its own
//! sibling module (`add`, `apply`, `backup`, `config`, `console`,
//! `delete`, `gen_autocomplete`, `gen_man`, `get`, `log`, `migrate`,
//! `power`, `restore`, `run`, `upgrade`) so each file covers one
//! slice of the user-facing tree.

use clap::{Command, arg};

mod add;
mod apply;
mod backup;
mod config;
mod console;
mod delete;
mod gen_autocomplete;
mod gen_man;
pub(crate) mod get;
mod log;
pub(crate) mod manpage;
mod migrate;
mod power;
mod restore;
mod run;
mod upgrade;

const CLI_TERM_WIDTH: usize = 100;

/// Shared help text for arguments that accept xnames, NIDs, or hostlist expressions.
pub(super) const HOSTLIST_HELP: &str = "Xnames, NIDs, or a hostlist expression.\n\
  eg: 'x1003c1s7b0n0,x1003c1s7b0n1', 'nid001313,nid001314',\n\
  'x1003c1s7b0n[0-1],x1003c1s7b1n0', 'nid00131[0-9]'";

/// Standard `-o/--output {table,json}` flag with `table` as default.
/// Mutating commands consume the result via
/// `crate::output::action_result::print`; read commands wire it
/// into their per-resource renderer in `crate::output::*`.
pub(super) fn output_flag() -> clap::Arg {
  arg!(-o --output <FORMAT> "Output format")
    .value_parser(["table", "json"])
    .default_value("table")
}

/// Long-only `--output {table,json}` variant for subcommands that
/// already use `-o` for something else (eg. `apply template --operation`
/// or `apply sat-file --overwrite-configuration`).
pub(super) fn output_flag_long_only() -> clap::Arg {
  arg!(--output <FORMAT> "Output format")
    .value_parser(["table", "json"])
    .default_value("table")
}

/// Standard `-d/--dry-run` flag for mutating verbs. Use this for every
/// new mutating leaf so the short alias `-d` stays reserved for dry-run
/// across the CLI. Verbs needing a custom help message (e.g. `upgrade`,
/// `delete configurations`) declare their flag inline rather than
/// through this helper.
pub(super) fn dry_run_flag() -> clap::Arg {
  arg!(-d --"dry-run" "Simulate the operation without making changes")
    .action(clap::ArgAction::SetTrue)
}

/// Build the clap CLI command tree for manta.
pub fn build_cli() -> Command {
  // Hard-coded "manta" rather than `env!("CARGO_PKG_NAME")` because the
  // package is `manta-cli` but the produced binary is `manta` (see the
  // `[[bin]]` block in this crate's Cargo.toml). Using the package name
  // would render wrong help text and produce wrong completion-script
  // names.
  Command::new("manta")
    .term_width(CLI_TERM_WIDTH)
    .version(env!("CARGO_PKG_VERSION"))
    .arg_required_else_help(true)
    .arg(
      // `.global(true)` propagates the flag to every subcommand so
      // users can place `--site <name>` anywhere on the command
      // line, including after the subcommand name (e.g.
      // `manta get sessions --site prealps`). main.rs reads the
      // value off the top-level matches via clap's automatic
      // promotion — no per-subcommand parsing is required.
      arg!(--site <SITE_NAME> "Override the active site for this invocation")
        .required(false)
        .global(true),
    )
    .subcommand(config::subcommand_config())
    .subcommand(get::subcommand_get())
    .subcommand(add::subcommand_add())
    .subcommand(apply::subcommand_apply())
    .subcommand(delete::subcommand_delete())
    .subcommand(migrate::subcommand_migrate())
    .subcommand(backup::subcommand_backup())
    .subcommand(restore::subcommand_restore())
    .subcommand(run::subcommand_run())
    .subcommand(power::subcommand_power())
    .subcommand(log::subcommand_log())
    .subcommand(console::subcommand_console())
    .subcommand(gen_autocomplete::subcommand_gen_autocomplete())
    .subcommand(gen_man::subcommand_gen_man())
    .subcommand(upgrade::subcommand_upgrade())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn site_flag_is_optional() {
    let _matches =
      build_cli().try_get_matches_from(["manta", "get", "sessions", "--help"]);
    let result = build_cli().try_get_matches_from(["manta", "--version"]);
    assert!(
      result.is_err(),
      "expected DisplayVersion error for --version"
    );
  }

  #[test]
  fn site_flag_accepted_before_subcommand() {
    let result = build_cli()
      .try_get_matches_from(["manta", "--site", "alps", "get", "--help"]);
    match result {
      Err(e) => assert_eq!(
        e.kind(),
        clap::error::ErrorKind::DisplayHelp,
        "expected DisplayHelp, got: {e}"
      ),
      Ok(_) => panic!("--help should cause an early exit"),
    }
  }

  #[test]
  fn site_flag_value_is_extractable() {
    let matches = build_cli()
      .get_matches_from(["manta", "--site", "prealps", "config", "show"]);
    let site = matches.get_one::<String>("site");
    assert_eq!(site.map(String::as_str), Some("prealps"));
  }

  #[test]
  fn site_flag_absent_returns_none() {
    let matches = build_cli().get_matches_from(["manta", "config", "show"]);
    let site = matches.get_one::<String>("site");
    assert!(site.is_none());
  }

  #[test]
  fn site_flag_accepted_after_subcommand_at_top() {
    // After `.global(true)`: `--site` works following any subcommand,
    // not just at the position before it.
    let matches = build_cli()
      .get_matches_from(["manta", "get", "sessions", "--site", "prealps"]);
    let site = matches.get_one::<String>("site");
    assert_eq!(site.map(String::as_str), Some("prealps"));
  }

  #[test]
  fn site_flag_accepted_after_nested_subcommand() {
    // Two-level subcommand path (`apply boot nodes …`) still surfaces
    // the global value back through the top-level matches.
    let matches = build_cli().get_matches_from([
      "manta",
      "apply",
      "boot",
      "nodes",
      "--site",
      "alpsb",
      "--boot-image",
      "abc-123",
      "x1000c0s0b0n0",
    ]);
    let site = matches.get_one::<String>("site");
    assert_eq!(site.map(String::as_str), Some("alpsb"));
  }
}
