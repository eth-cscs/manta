//! Implements the `manta gen-man` command.
//!
//! Mirrors what `build.rs` does at compile time when
//! `MANTA_REGENERATE_DOCS=1` is set, but driven from the live clap
//! tree at runtime so end users can produce an up-to-date page for
//! whichever binary they have installed — without needing the manta
//! source tree.

use std::ffi::OsString;
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{Context, Error, Result, anyhow};
use clap::{ArgMatches, Command};
use serde_json::json;

use crate::build::manpage;
use crate::common::app_context::AppContext;
use crate::output::action_result;

/// Dispatch `manta gen-man`.
///
/// Like `gen-autocomplete`, this handler does NOT call
/// `get_api_token` — installing man pages is purely local.
pub async fn handle_gen_man(
  cli_gen_man: &ArgMatches,
  _ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let path = cli_gen_man.get_one::<PathBuf>("path").cloned();
  let output_opt = cli_gen_man.get_one::<String>("output").map(String::as_str);

  let cli = crate::build::build_cli();
  exec(cli, path, output_opt)
}

/// Generate and install the consolidated `manta.1` man page.
///
/// Resolves the destination directory (`target` or the XDG user
/// `man1` default — see [`default_user_man_dir`]), then defers to
/// [`manpage::render_consolidated`] to walk the clap tree and emit a
/// single `manta.1` covering every subcommand.
///
/// # Errors
///
/// - No `target` was given and the XDG / `$HOME` environment is
///   insufficient to compute a default install location.
/// - The destination directory could not be created.
/// - The output file could not be opened, or
///   [`manpage::render_consolidated`] failed to write the page.
pub fn exec(
  cli: Command,
  target: Option<PathBuf>,
  output_opt: Option<&str>,
) -> Result<()> {
  let dest = match target {
    Some(p) => p,
    None => default_user_man_dir(
      std::env::var_os("XDG_DATA_HOME"),
      std::env::var_os("HOME"),
    )?,
  };

  fs::create_dir_all(&dest).with_context(|| {
    format!("failed to create destination directory {}", dest.display())
  })?;

  let path = dest.join("manta.1");
  let mut f = File::create(&path)
    .with_context(|| format!("failed to create {}", path.display()))?;
  manpage::render_consolidated(cli, &mut f).with_context(|| {
    format!("failed to render manta.1 into {}", path.display())
  })?;

  action_result::print_with_data(
    &format!("Installed manta.1 to {}.", dest.display()),
    &json!({"path": dest.to_string_lossy()}),
    output_opt,
  )?;

  Ok(())
}

/// `$XDG_DATA_HOME/man/man1`, with the XDG fallback to
/// `$HOME/.local/share`. Pure: env values are passed in rather than
/// read here so tests don't have to mutate global state.
fn default_user_man_dir(
  xdg_data_home: Option<OsString>,
  home: Option<OsString>,
) -> Result<PathBuf> {
  let base = xdg_data_home
    .map(PathBuf::from)
    .or_else(|| home.map(|h| PathBuf::from(h).join(".local").join("share")))
    .ok_or_else(|| {
      anyhow!(
        "could not determine $XDG_DATA_HOME or $HOME; \
         pass an explicit --path"
      )
    })?;
  Ok(base.join("man").join("man1"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn xdg_data_home_takes_precedence_over_home() {
    let p = default_user_man_dir(
      Some("/tmp/xdg-test".into()),
      Some("/Users/alice".into()),
    )
    .unwrap();
    assert_eq!(p, PathBuf::from("/tmp/xdg-test/man/man1"));
  }

  #[test]
  fn home_fallback_used_when_xdg_unset() {
    let p = default_user_man_dir(None, Some("/Users/bob".into())).unwrap();
    assert_eq!(p, PathBuf::from("/Users/bob/.local/share/man/man1"));
  }

  #[test]
  fn error_when_both_unset() {
    let r = default_user_man_dir(None, None);
    assert!(r.is_err());
  }
}
