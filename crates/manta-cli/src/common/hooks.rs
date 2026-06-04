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

#[cfg(test)]
mod tests {
  //! Hooks run arbitrary shell commands as part of `apply sat-file`,
  //! `backup vcluster`, and `restore vcluster`. The behavioural
  //! contract: empty/missing inputs fail loudly, non-zero exit codes
  //! propagate as errors, and existence + executable-bit are checked
  //! in that order. Tests are Unix-only — the executable-bit check
  //! relies on Unix file modes.

  use super::*;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;

  #[test]
  fn run_hook_rejects_none() {
    let err = run_hook(None).unwrap_err().to_string();
    assert!(err.contains("Hook command is empty"), "got: {err}");
  }

  #[test]
  fn check_hook_perms_rejects_none() {
    let err = check_hook_perms(None).unwrap_err().to_string();
    assert!(err.contains("Hook command is empty"), "got: {err}");
  }

  #[test]
  fn check_hook_perms_rejects_missing_file() {
    let err = check_hook_perms(Some("/nonexistent/path/to/hook.sh"))
      .unwrap_err()
      .to_string();
    assert!(err.contains("does not exist"), "got: {err}");
  }

  #[test]
  fn check_hook_perms_rejects_non_executable_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not-executable.sh");
    fs::write(&path, "#!/bin/sh\necho hi\n").unwrap();
    // mode 0o644: rw-r--r-- — readable but not executable
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    let err = check_hook_perms(Some(path.to_str().unwrap()))
      .unwrap_err()
      .to_string();
    assert!(err.contains("not executable"), "got: {err}");
  }

  #[test]
  fn check_hook_perms_accepts_executable_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hook.sh");
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(check_hook_perms(Some(path.to_str().unwrap())).is_ok());
  }

  #[test]
  fn check_hook_perms_parses_first_token_as_program_name() {
    // A hook command like "/path/to/script.sh --arg foo" should
    // only validate "/path/to/script.sh", not the full string.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hook.sh");
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();

    let cmd = format!("{} --arg foo --another bar", path.display());
    assert!(check_hook_perms(Some(&cmd)).is_ok());
  }

  #[test]
  fn run_hook_returns_zero_on_success() {
    let code = run_hook(Some("true")).unwrap();
    assert_eq!(code, 0);
  }

  #[test]
  fn run_hook_errors_on_non_zero_exit() {
    // `false` exits with code 1.
    let err = run_hook(Some("false")).unwrap_err().to_string();
    assert!(err.contains("return code=1"), "got: {err}");
    assert!(err.contains("hook failed"), "got: {err}");
  }

  #[test]
  fn run_hook_propagates_specific_exit_code() {
    // `sh -c 'exit 42'` exits with code 42; the error message should
    // surface the actual code so operators can debug.
    let err = run_hook(Some("sh -c 'exit 42'")).unwrap_err().to_string();
    assert!(err.contains("return code=42"), "got: {err}");
  }
}
