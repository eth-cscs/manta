use std::path::Path;

use anyhow::Error;
use execute::{shell, Execute};
use is_executable::IsExecutable;

/// Executes the hook using a subshell. stdout and stderr are redirected to the main process stdout
/// returns Ok(exit_code) or Err() with the description of the error
pub async fn run_hook(hook_opt: Option<&str>) -> Result<i32, Error> {
  let mut command = shell(&hook_opt.unwrap());
  // command.stdout(Stdio::piped());
  let output = command.execute_output().unwrap();
  // println!("{}", String::from_utf8(output.stdout).unwrap());
  if let Some(exit_code) = output.status.code() {
    if exit_code != 0 {
      return Err(Error::msg(format!(
        "Error: the hook failed with return code={}. I will not continue.",
        exit_code
      )));
    } else {
      return Ok(exit_code);
    }
  } else {
    return Err(Error::msg(
      "Error: the hook was interrupted, will not continue.",
    ));
  }
}

/// Checks that the hook exists and is executable
/// returns Ok if all good, an error message otherwise
pub async fn check_hook_perms(hook_opt: Option<&str>) -> Result<(), Error> {
  if hook_opt.is_some() {
    let program_name = hook_opt.unwrap().split(" ").nth(0).unwrap();
    let hookpath = Path::new(program_name);
    if !&hookpath.exists() {
      return Err(Error::msg("Error: the hook file does not exist."));
    } else if !&hookpath.is_executable() {
      return Err(Error::msg(
        "Error: the hook file is not executable does not exist.",
      ));
    } else {
      return Ok(());
    }
  } else {
    return Err(Error::msg("Hook is empty"));
  }
}
