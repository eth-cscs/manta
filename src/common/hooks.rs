use std::{error::Error, path::Path};

use execute::{shell, Execute};
use is_executable::IsExecutable;

/// Executes the hook using a subshell. stdout and stderr are redirected to the main process stdout
/// returns Ok(exit_code) or Err() with the description of the error
pub async fn run_hook(hook: Option<&String>) -> Result<i32, Box<dyn Error>> {
  let mut command = shell(&hook.unwrap());
  // command.stdout(Stdio::piped());
  let output = command.execute_output().unwrap();
  // println!("{}", String::from_utf8(output.stdout).unwrap());
  if let Some(exit_code) = output.status.code() {
    if exit_code != 0 {
      Err("The hook failed with return code {}")?;
      eprintln!(
        "Error: the hook failed with return code={}. I will not continue.",
        exit_code
      );
    } else {
      return Ok(exit_code);
    }
  } else {
    Err("Error: the hook was interrupted, will not continue.")?;
  }
  println!("Done with the hook.");
  Ok(0)
}

/// Checks that the hook exists and is executable
/// returns Ok if all good, an error message otherwise
pub async fn check_hook_perms(
  hook: Option<&String>,
) -> Result<(), Box<dyn Error>> {
  if hook.is_some() {
    let program_name = hook.unwrap().split(" ").nth(0).unwrap();
    let hookpath = Path::new(program_name);
    if !&hookpath.exists() {
      Err("Error: the hook file does not exist.")?;
    } else if !&hookpath.is_executable() {
      Err("Error: the hook file is not executable does not exist.")?;
    } else {
      return Ok(());
    }
  } else {
    Err("Hook is empty")?;
  }
  Ok(())
}
