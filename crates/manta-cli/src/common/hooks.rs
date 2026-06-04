//! Pre-/post-hook execution: validate that the configured hook path
//! is an executable shell command, then run it via a subshell and
//! return its exit code. Used by `apply sat-file`, `backup vcluster`,
//! and `restore vcluster`.

use std::path::Path;

use anyhow::{Error, anyhow, bail};
use execute::{Execute, shell};
use is_executable::IsExecutable;

/// Executes the hook using a subshell. stdout and stderr are redirected to the main process stdout
/// returns Ok(exit_code) or Err() with the description of the error
pub fn run_hook(hook_opt: Option<&str>) -> Result<i32, Error> {
  let hook = hook_opt.ok_or_else(|| anyhow!("Hook command is empty"))?;
  let mut command = shell(hook);
  let output = command.execute_output()?;
  if let Some(exit_code) = output.status.code() {
    if exit_code != 0 {
      bail!(
        "Error: the hook failed with return code={exit_code}. \
         I will not continue."
      );
    }
    Ok(exit_code)
  } else {
    bail!("Error: the hook was interrupted, will not continue.")
  }
}

/// Checks that the hook exists and is executable
/// returns Ok if all good, an error message otherwise
pub fn check_hook_perms(hook_opt: Option<&str>) -> Result<(), Error> {
  let hook = hook_opt.ok_or_else(|| anyhow!("Hook command is empty"))?;
  let program_name = hook
    .split(' ')
    .next()
    .ok_or_else(|| anyhow!("Could not parse hook command"))?;
  let hookpath = Path::new(program_name);
  if !hookpath.exists() {
    bail!("Error: the hook file does not exist.")
  } else if !hookpath.is_executable() {
    bail!("Error: the hook file is not executable.")
  } else {
    Ok(())
  }
}
