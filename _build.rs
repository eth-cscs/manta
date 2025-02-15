use clap_complete::{generate_to, Shell};
use cli::build_cli;
use std::io::Error;

#[path = "src/cli/build.rs"]
mod cli;

fn main() -> Result<(), Error> {
    let outdir = "autocomplete_shell_scripts";

    let mut cmd = build_cli();

    let path = generate_to(Shell::Bash, &mut cmd, env!("CARGO_PKG_NAME"), outdir)?;

    println!("cargo:error=completion file is not reliable for tenants");
    println!("cargo:info=completion file is generated: {:?}", path);

    let path = generate_to(Shell::Zsh, &mut cmd, env!("CARGO_PKG_NAME"), outdir)?;

    println!("cargo:error=completion file is not reliable for tenants");
    println!("cargo:info=completion file is generated: {:?}", path);

    let path = generate_to(Shell::Fish, &mut cmd, env!("CARGO_PKG_NAME"), outdir)?;

    println!("cargo:error=completion file is not reliable for tenants");
    println!("cargo:warning=completion file is generated: {:?}", path);

    let path = generate_to(Shell::Elvish, &mut cmd, env!("CARGO_PKG_NAME"), outdir)?;

    println!("cargo:error=completion file is not reliable for tenants");
    println!("cargo:warning=completion file is generated: {:?}", path);

    Ok(())
}
