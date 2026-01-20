use std::{env, io, path::PathBuf};

use anyhow::Error;

use clap::{ArgMatches, Command};
use clap_complete::{generate, generate_to};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn process_subcommand(
  mut cli: Command,
  cli_config_generate_autocomplete: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shell_opt: Option<String> =
    cli_config_generate_autocomplete.get_one("shell").cloned();

  let path_opt: Option<PathBuf> =
    cli_config_generate_autocomplete.get_one("path").cloned();

  let shell = if let Some(shell) = shell_opt {
    shell.to_ascii_uppercase()
  } else {
    let shell_ostring =
      PathBuf::from(env::var_os("SHELL").expect("$SHELL env missing"))
        .file_name()
        .unwrap()
        .to_ascii_uppercase();

    shell_ostring
      .into_string()
      .expect("Could not convert shell name to string")
  };

  let shell_gen = match shell.as_str() {
    "BASH" => clap_complete::Shell::Bash,
    "ZSH" => clap_complete::Shell::Zsh,
    "FISH" => clap_complete::Shell::Fish,
    _ => {
      eprintln!("ERROR - Shell '{shell}' not supported",);
      std::process::exit(1);
    }
  };

  if let Some(path) = path_opt {
    // Destination path defined
    log::info!(
      "Generating shell autocomplete for '{}' to '{}'",
      shell,
      path.display()
    );
    generate_to(shell_gen, &mut cli, env!("CARGO_PKG_NAME"), path)?;
  } else {
    // Destination path not defined - print to stdout
    log::info!("Generating shell autocomplete for '{}'", shell);
    generate(
      shell_gen,
      &mut cli,
      env!("CARGO_PKG_NAME"),
      &mut io::stdout(),
    );
  }

  Ok(())
}
