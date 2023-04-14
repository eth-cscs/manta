use clap_complete::{generate_to, shells::Bash};
use std::io::Error;

include!("src/cli/build.rs");

fn main() -> Result<(), Error> {
    let outdir = "target";

    println!("0");

    let mut cmd = build_cli(None);
    println!("1");
    let path = generate_to(
        Bash, &mut cmd, // We need to specify what generator to use
        "manta",  // We need to specify the bin name manually
        outdir,   // We need to specify where to write to
    )?;

    println!("cargo:warning=completion file is generated: {:?}", path);

    Ok(())
}
