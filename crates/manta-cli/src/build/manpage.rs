//! Consolidated man-page renderer.
//!
//! `clap_mangen::generate_to` emits one `.1` file per Command node in
//! the clap tree — for manta that's 84 files. This module produces a
//! single `manta.1` instead: the standard top-level sections, followed
//! by a SUBCOMMANDS section with one `.SS` subsection per
//! (sub)subcommand carrying its description and option table inline.
//!
//! The renderer is shared between `build.rs` (compile-time regen when
//! `MANTA_REGENERATE_DOCS=1`) and the runtime `manta gen-man` command,
//! both of which write the result to `<dir>/manta.1`.

use std::io::Write;

use clap::{Arg, ArgAction, Command};
use clap_mangen::Man;
use clap_mangen::roff::{Inline, Roff, bold, italic, roman};

/// Render the consolidated manta man page into `w`.
///
/// `cli` is consumed because clap-mangen mutates the tree during
/// rendering. The output is a complete roff document: NAME /
/// SYNOPSIS / DESCRIPTION / OPTIONS at the top, then a single
/// SUBCOMMANDS section walking the subcommand tree depth-first.
///
/// # Errors
///
/// Returns `Err` if any write to `w` fails — usually a broken pipe
/// when stdout is gone.
pub fn render_consolidated(
  cli: Command,
  w: &mut dyn Write,
) -> std::io::Result<()> {
  let mut cli = cli.disable_help_subcommand(true);
  cli.build();

  // Top-level sections — reuse clap_mangen for the standard
  // NAME / SYNOPSIS / DESCRIPTION / OPTIONS layout.
  let top = Man::new(cli.clone());
  top.render_title(w)?;
  top.render_name_section(w)?;
  top.render_synopsis_section(w)?;
  top.render_description_section(w)?;
  if has_visible_args(&cli) {
    top.render_options_section(w)?;
  }

  // One SUBCOMMANDS section covering every (sub)subcommand,
  // depth-first.
  let mut sub_roff = Roff::default();
  sub_roff.control("SH", ["SUBCOMMANDS"]);
  let top_name = cli
    .get_bin_name()
    .unwrap_or_else(|| cli.get_name())
    .to_owned();
  walk_subcommands(&cli, &mut sub_roff, &[top_name.as_str()]);
  sub_roff.to_writer(w)?;

  Ok(())
}

fn walk_subcommands(cmd: &Command, roff: &mut Roff, path: &[&str]) {
  for sub in cmd.get_subcommands().filter(|s| !s.is_hide_set()) {
    let mut sub_path: Vec<&str> = path.to_vec();
    sub_path.push(sub.get_name());
    let heading = sub_path.join(" ");

    roff.control("SS", [heading.as_str()]);
    if let Some(about) = sub.get_long_about().or_else(|| sub.get_about()) {
      for line in about.to_string().lines() {
        if line.trim().is_empty() {
          roff.control("PP", []);
        } else {
          roff.text([roman(line)]);
        }
      }
    }
    render_args(sub, roff);

    walk_subcommands(sub, roff, &sub_path);
  }
}

/// Emit the option + positional table for `cmd` as `.TP` blocks.
/// Mirrors the subset of `clap_mangen::render::options` we need
/// (that helper is `pub(crate)`, so we can't reuse it directly).
fn render_args(cmd: &Command, roff: &mut Roff) {
  let opts: Vec<&Arg> = cmd
    .get_arguments()
    .filter(|a| !a.is_hide_set())
    .filter(|a| !a.is_positional())
    .collect();
  let positionals: Vec<&Arg> = cmd.get_positionals().collect();
  if opts.is_empty() && positionals.is_empty() {
    return;
  }

  for opt in opts {
    let (lhs, rhs) = option_markers(opt);
    let mut tag: Vec<Inline> = Vec::new();
    match (opt.get_short(), opt.get_long()) {
      (Some(short), Some(long)) => {
        tag.push(roman(lhs));
        tag.push(bold(format!("-{short}")));
        tag.push(roman(", "));
        tag.push(bold(format!("--{long}")));
        tag.push(roman(rhs));
      }
      (Some(short), None) => {
        tag.push(roman(lhs));
        tag.push(bold(format!("-{short}")));
        tag.push(roman(rhs));
      }
      (None, Some(long)) => {
        tag.push(roman(lhs));
        tag.push(bold(format!("--{long}")));
        tag.push(roman(rhs));
      }
      (None, None) => continue,
    }
    if matches!(opt.get_action(), ArgAction::Count) {
      tag.push(roman("..."));
    }
    if let Some(value_names) = opt.get_value_names() {
      tag.push(roman(" "));
      tag.push(italic(format!("<{}>", value_names.join(" "))));
    }

    roff.control("TP", []);
    roff.text(tag);
    if let Some(help) = opt.get_long_help().or_else(|| opt.get_help()) {
      roff.text([roman(help.to_string())]);
    }
  }

  for arg in positionals {
    let (lhs, rhs) = option_markers(arg);
    let mut tag: Vec<Inline> = Vec::new();
    tag.push(roman(lhs));
    if let Some(value_names) = arg.get_value_names() {
      tag.push(italic(value_names.join(" ")));
    } else {
      tag.push(italic(arg.get_id().as_str().to_owned()));
    }
    tag.push(roman(rhs));

    roff.control("TP", []);
    roff.text(tag);
    if let Some(help) = arg.get_long_help().or_else(|| arg.get_help()) {
      roff.text([roman(help.to_string())]);
    }
  }
}

fn option_markers(arg: &Arg) -> (&'static str, &'static str) {
  if arg.is_required_set() {
    ("", "")
  } else {
    ("[", "]")
  }
}

fn has_visible_args(cmd: &Command) -> bool {
  cmd.get_arguments().any(|a| !a.is_hide_set())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{Command, arg};

  fn tiny_cli() -> Command {
    Command::new("toy")
      .about("Toy CLI for testing the consolidated man page")
      .arg(arg!(-v --verbose "Enable verbose output"))
      .subcommand(
        Command::new("get").about("Read-only queries").subcommand(
          Command::new("sessions")
            .about("List sessions")
            .arg(arg!(-n --name <NAME> "Session name filter")),
        ),
      )
  }

  #[test]
  fn render_consolidated_emits_top_level_and_subcommands() {
    let mut buf: Vec<u8> = Vec::new();
    render_consolidated(tiny_cli(), &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // Top-level page sections present. `roff` only quotes args
    // that contain spaces, so single-word `.SH` headers are bare.
    assert!(text.contains(".SH NAME"), "missing NAME header: {text}");
    assert!(
      text.contains(".SH OPTIONS"),
      "missing OPTIONS header: {text}"
    );

    // SUBCOMMANDS section appears exactly once.
    assert_eq!(
      text.matches(".SH SUBCOMMANDS").count(),
      1,
      "expected one SUBCOMMANDS header: {text}"
    );

    // Each (sub)subcommand becomes an `.SS` subsection prefixed with
    // the full verb path. These DO get quoted because the heading
    // contains spaces.
    assert!(
      text.contains(".SS \"toy get\""),
      "missing top-level subcommand SS: {text}"
    );
    assert!(
      text.contains(".SS \"toy get sessions\""),
      "missing nested SS: {text}"
    );
  }

  #[test]
  fn render_consolidated_inlines_subcommand_option_tables() {
    let mut buf: Vec<u8> = Vec::new();
    render_consolidated(tiny_cli(), &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // The subcommand's `--name` arg shows up as a TP block under its
    // section. roff escapes hyphens as `\-`, so the literal flag in
    // the output is `\-\-name`.
    assert!(
      text.contains(r"\-\-name"),
      "subcommand option missing from page: {text}"
    );
    assert!(
      text.contains("Session name filter"),
      "subcommand help missing from page: {text}"
    );
  }

  #[test]
  fn render_consolidated_handles_command_with_no_subcommands() {
    let cli = Command::new("solo")
      .about("No subcommands here")
      .arg(arg!(-f --force "Skip confirmation"));
    let mut buf: Vec<u8> = Vec::new();
    render_consolidated(cli, &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // The SUBCOMMANDS header is still emitted (empty), so the page
    // structure stays predictable. No `.SS` rows follow it.
    assert!(text.contains(".SH SUBCOMMANDS"));
    assert!(!text.contains(".SS "), "no SS expected: {text}");
  }
}
