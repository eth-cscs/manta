use clap_complete::{generate_to, shells::Bash};
use std::{env, io::Error};

include!("src/cli/build.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = build_cli(None, Vec::new());
    let path = generate_to(
        Bash, &mut cmd, // We need to specify what generator to use
        "manta",  // We need to specify the bin name manually
        outdir,   // We need to specify where to write to
    )?;

    println!("cargo:eror=completion file is not reliable for tenants");
    println!("cargo:warning=completion file is generated: {:?}", path);

    Ok(())
}
