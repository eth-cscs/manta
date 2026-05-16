use std::path::Path;

use execute::{Execute, shell};
use is_executable::IsExecutable;
use manta_backend_dispatcher::error::Error;

/// Executes the hook using a subshell. stdout and stderr are redirected to the main process stdout
/// returns Ok(exit_code) or Err() with the description of the error
pub fn run_hook(hook_opt: Option<&str>) -> Result<i32, Error> {
  let hook =
    hook_opt.ok_or_else(|| Error::HookError("Hook command is empty".to_string()))?;
  let mut command = shell(hook);
  let output = command.execute_output()?;
  if let Some(exit_code) = output.status.code() {
    if exit_code != 0 {
      Err(Error::HookError(format!(
        "Error: the hook failed with return code={}. \
         I will not continue.",
        exit_code
      )))
    } else {
      Ok(exit_code)
    }
  } else {
    Err(Error::HookError(
      "Error: the hook was interrupted, \
       will not continue."
        .to_string(),
    ))
  }
}

/// Checks that the hook exists and is executable
/// returns Ok if all good, an error message otherwise
pub fn check_hook_perms(hook_opt: Option<&str>) -> Result<(), Error> {
  let hook =
    hook_opt.ok_or_else(|| Error::HookError("Hook command is empty".to_string()))?;
  let program_name = hook.split(' ').next().ok_or_else(|| {
    Error::HookError("Could not parse hook command".to_string())
  })?;
  let hookpath = Path::new(program_name);
  if !hookpath.exists() {
    Err(Error::HookError(
      "Error: the hook file does not exist.".to_string(),
    ))
  } else if !hookpath.is_executable() {
    Err(Error::HookError(
      "Error: the hook file is not executable.".to_string(),
    ))
  } else {
    Ok(())
  }
}
