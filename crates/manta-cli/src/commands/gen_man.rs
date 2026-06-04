//! Implements the `manta gen-man` command.
//!
//! Mirrors what `build.rs` does at compile time when
//! `MANTA_REGENERATE_DOCS=1` is set, but driven from the live clap
//! tree at runtime so end users can produce up-to-date pages for
//! whichever binary they have installed — without needing the manta
//! source tree.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::Command;
use clap_mangen::generate_to;
use serde_json::json;

use crate::output::action_result;

/// Generate and install the manta man pages.
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

  generate_to(cli, &dest).with_context(|| {
    format!("failed to generate man pages into {}", dest.display())
  })?;

  let count = count_files(&dest)?;

  action_result::print_with_data(
    &format!("Installed {count} man page(s) to {}.", dest.display()),
    &json!({"path": dest.to_string_lossy(), "count": count}),
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

/// Count `.1` files in `dir` so we can report a meaningful "Installed
/// N pages" message. `clap_mangen::generate_to` returns `()` (no
/// per-file accounting), so we read it back from the filesystem.
fn count_files(dir: &Path) -> Result<usize> {
  let entries = fs::read_dir(dir)
    .with_context(|| format!("failed to read {}", dir.display()))?;
  let mut n = 0;
  for entry in entries {
    let entry = entry?;
    if entry.file_type()?.is_file()
      && entry.path().extension().and_then(|e| e.to_str()) == Some("1")
    {
      n += 1;
    }
  }
  Ok(n)
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

  #[test]
  fn count_files_counts_only_dot1_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("foo.1"), "x").unwrap();
    fs::write(dir.path().join("bar.1"), "y").unwrap();
    fs::write(dir.path().join("baz.txt"), "z").unwrap();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    assert_eq!(count_files(dir.path()).unwrap(), 2);
  }
}
