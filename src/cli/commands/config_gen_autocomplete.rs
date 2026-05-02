use clap::{ArgMatches, Command};
use clap_complete::{generate, generate_to};

use std::{env, io, path::PathBuf};

use anyhow::{bail, Error};

/// Generate shell auto-completion scripts.
pub fn exec(
  cli: Command,
  cli_config_generate_autocomplete: &ArgMatches,
) -> Result<(), Error> {
  let shell_opt: Option<String> =
    cli_config_generate_autocomplete.get_one("shell").cloned();

  let path_opt: Option<PathBuf> =
    cli_config_generate_autocomplete.get_one("path").cloned();

  gen_autocomplete(cli, shell_opt, path_opt)
}

fn gen_autocomplete(
  mut cli: clap::Command,
  shell_opt: Option<String>,
  path_opt: Option<PathBuf>,
) -> Result<(), Error> {
  let shell = if let Some(shell) = shell_opt {
    shell.to_ascii_uppercase()
  } else {
    let shell_ostring = PathBuf::from(
      env::var_os("SHELL").ok_or_else(|| Error::msg("$SHELL env missing"))?,
    )
    .file_name()
    .map(|v| v.to_ascii_uppercase())
    .ok_or_else(|| Error::msg("Could not determine shell from $SHELL env"))?;

    shell_ostring.into_string().map_err(|os_str| {
      anyhow::anyhow!(
        "Could not convert shell name to string: '{}'",
        os_str.to_string_lossy()
      )
    })?
  };

  let shell_gen = match shell.as_str() {
    "BASH" => clap_complete::Shell::Bash,
    "ZSH" => clap_complete::Shell::Zsh,
    "FISH" => clap_complete::Shell::Fish,
    _ => {
      bail!("Shell '{}' not supported", shell);
    }
  };

  if let Some(path) = path_opt {
    // Destination path defined
    tracing::info!(
      "Generating shell autocomplete for '{}' to '{}'",
      shell,
      path.display()
    );

    generate_to(shell_gen, &mut cli, env!("CARGO_PKG_NAME"), path)?;
  } else {
    // Destination path not defined - print to stdout
    tracing::info!("Generating shell autocomplete for '{}'", shell);

    generate(
      shell_gen,
      &mut cli,
      env!("CARGO_PKG_NAME"),
      &mut io::stdout(),
    );
  }

  Ok(())
}
