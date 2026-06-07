//! Implements the `manta gen-autocomplete` command.
//!
//! Default behaviour is **install** — when no `--path` is given, the
//! script is written into the shell's standard XDG user completion
//! directory. `--path <DIR>` overrides the install location, and
//! `--print` emits the script to stdout without touching the
//! filesystem (useful for `eval "$(manta gen-autocomplete --shell zsh
//! --print)"` style dynamic loading).

use std::{env, ffi::OsString, fs, io, path::PathBuf};

use anyhow::{Context, Error, Result, anyhow, bail};
use clap::{ArgMatches, Command};
use clap_complete::{Shell, generate, generate_to};
use serde_json::json;

use crate::common::clap_ext::ArgMatchesExt;
use crate::output::action_result;

/// Generate (and by default install) the shell completion script.
pub fn exec(mut cli: Command, cli_gen_autocomplete: &ArgMatches) -> Result<()> {
  let shell_opt: Option<String> =
    cli_gen_autocomplete.opt_str("shell").map(str::to_owned);
  let path_opt: Option<PathBuf> = cli_gen_autocomplete.get_one("path").cloned();
  let print: bool = cli_gen_autocomplete.get_flag("print");
  let output_opt: Option<&str> = cli_gen_autocomplete.opt_str("output");

  let shell = resolve_shell(shell_opt)?;

  if print {
    generate(shell, &mut cli, "manta", &mut io::stdout());
    return Ok(());
  }

  let dir = match path_opt {
    Some(p) => p,
    None => default_install_dir(
      shell,
      env::var_os("XDG_DATA_HOME"),
      env::var_os("XDG_CONFIG_HOME"),
      env::var_os("HOME"),
    )?,
  };

  fs::create_dir_all(&dir).with_context(|| {
    format!("failed to create destination directory {}", dir.display())
  })?;

  let path =
    generate_to(shell, &mut cli, "manta", &dir).with_context(|| {
      format!("failed to write completion script under {}", dir.display())
    })?;

  let extra = post_install_hint(shell, &dir);
  let message = match extra {
    Some(hint) => format!(
      "Installed {} completion to {}.\n{}",
      shell,
      path.display(),
      hint
    ),
    None => format!("Installed {} completion to {}.", shell, path.display()),
  };
  action_result::print_with_data(
    &message,
    &json!({"path": path.to_string_lossy(), "shell": shell.to_string()}),
    output_opt,
  )?;

  Ok(())
}

fn resolve_shell(shell_opt: Option<String>) -> Result<Shell> {
  let name = if let Some(s) = shell_opt {
    s.to_ascii_lowercase()
  } else {
    let env_shell = env::var_os("SHELL")
      .ok_or_else(|| anyhow!("$SHELL not set; pass --shell explicitly"))?;
    PathBuf::from(env_shell)
      .file_name()
      .ok_or_else(|| anyhow!("could not parse $SHELL"))?
      .to_string_lossy()
      .to_ascii_lowercase()
  };
  match name.as_str() {
    "bash" => Ok(Shell::Bash),
    "zsh" => Ok(Shell::Zsh),
    "fish" => Ok(Shell::Fish),
    other => bail!("shell '{other}' not supported (bash, zsh, fish)"),
  }
}

/// Standard XDG location for the chosen shell's user completions.
/// Pure — env values are passed in so tests don't have to mutate
/// global state.
//
// `home` is cloned three times below to thread it into the per-shell
// branches. Taking it by reference would cascade into `data_home` /
// `config_home`'s signatures too, which moves `xdg`. Net wins are
// three small `OsString` clones avoided once per CLI invocation —
// not worth the cascading API churn.
#[allow(clippy::needless_pass_by_value)]
fn default_install_dir(
  shell: Shell,
  xdg_data_home: Option<OsString>,
  xdg_config_home: Option<OsString>,
  home: Option<OsString>,
) -> Result<PathBuf, Error> {
  match shell {
    Shell::Bash => data_home(xdg_data_home, home.clone())
      .map(|d| d.join("bash-completion").join("completions")),
    Shell::Zsh => data_home(xdg_data_home, home.clone())
      .map(|d| d.join("zsh").join("site-functions")),
    Shell::Fish => config_home(xdg_config_home, home.clone())
      .map(|c| c.join("fish").join("completions")),
    other => bail!("shell '{other}' has no default install path"),
  }
}

fn data_home(xdg: Option<OsString>, home: Option<OsString>) -> Result<PathBuf> {
  xdg
    .map(PathBuf::from)
    .or_else(|| home.map(|h| PathBuf::from(h).join(".local").join("share")))
    .ok_or_else(|| {
      anyhow!(
        "could not determine $XDG_DATA_HOME or $HOME; \
         pass an explicit --path"
      )
    })
}

fn config_home(
  xdg: Option<OsString>,
  home: Option<OsString>,
) -> Result<PathBuf> {
  xdg
    .map(PathBuf::from)
    .or_else(|| home.map(|h| PathBuf::from(h).join(".config")))
    .ok_or_else(|| {
      anyhow!(
        "could not determine $XDG_CONFIG_HOME or $HOME; \
         pass an explicit --path"
      )
    })
}

/// Shells differ in how much setup the user must do after the script
/// is in place. zsh in particular needs the directory on `$fpath`
/// before `compinit` runs; bash and fish pick up the standard
/// XDG paths automatically.
fn post_install_hint(shell: Shell, dir: &std::path::Path) -> Option<String> {
  match shell {
    Shell::Zsh => Some(format!(
      "Note: ensure `{}` is on your $fpath before `compinit` runs.\n\
       Add this to ~/.zshrc:\n  fpath+=({})\n  autoload -Uz compinit && compinit",
      dir.display(),
      dir.display(),
    )),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_install_dir_bash_uses_xdg_data_home() {
    let p = default_install_dir(
      Shell::Bash,
      Some("/tmp/xdg-data".into()),
      None,
      None,
    )
    .unwrap();
    assert_eq!(
      p,
      PathBuf::from("/tmp/xdg-data/bash-completion/completions")
    );
  }

  #[test]
  fn default_install_dir_bash_falls_back_to_home() {
    let p =
      default_install_dir(Shell::Bash, None, None, Some("/Users/alice".into()))
        .unwrap();
    assert_eq!(
      p,
      PathBuf::from("/Users/alice/.local/share/bash-completion/completions")
    );
  }

  #[test]
  fn default_install_dir_zsh_lands_in_site_functions() {
    let p =
      default_install_dir(Shell::Zsh, None, None, Some("/Users/bob".into()))
        .unwrap();
    assert_eq!(
      p,
      PathBuf::from("/Users/bob/.local/share/zsh/site-functions")
    );
  }

  #[test]
  fn default_install_dir_fish_uses_xdg_config_home() {
    let p = default_install_dir(
      Shell::Fish,
      None,
      Some("/tmp/xdg-config".into()),
      None,
    )
    .unwrap();
    assert_eq!(p, PathBuf::from("/tmp/xdg-config/fish/completions"));
  }

  #[test]
  fn default_install_dir_fish_falls_back_to_dot_config() {
    let p =
      default_install_dir(Shell::Fish, None, None, Some("/Users/carol".into()))
        .unwrap();
    assert_eq!(p, PathBuf::from("/Users/carol/.config/fish/completions"));
  }

  #[test]
  fn default_install_dir_errors_when_xdg_and_home_both_unset() {
    let r = default_install_dir(Shell::Bash, None, None, None);
    assert!(r.is_err());
    let r = default_install_dir(Shell::Fish, None, None, None);
    assert!(r.is_err());
  }

  #[test]
  fn post_install_hint_warns_only_for_zsh() {
    let dir = std::path::Path::new("/tmp/x");
    assert!(post_install_hint(Shell::Zsh, dir).is_some());
    assert!(post_install_hint(Shell::Bash, dir).is_none());
    assert!(post_install_hint(Shell::Fish, dir).is_none());
  }
}
