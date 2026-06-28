//! Pre-/post-hook execution: validate that the configured hook path
//! is an executable shell command, then run it via a subshell and
//! return its exit code. Used by `apply sat-file`, `backup vcluster`,
//! and `restore vcluster`.

use std::path::Path;

use anyhow::{Error, anyhow, bail};
use execute::{Execute, shell};
use is_executable::IsExecutable;

/// Execute `hook_opt` as a shell command. stdout and stderr inherit
/// the parent process's streams.
///
/// # Errors
///
/// - `hook_opt` is `None`.
/// - The subshell could not be spawned (the `execute` crate surfaces
///   the OS error).
/// - The hook exited with a non-zero status (the error names the
///   exit code).
/// - The hook was killed by a signal (no exit code available).
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

/// Run a hook script if one was provided, prefixing user-visible
/// output and logs with `label` ("pre" / "post" by convention).
///
/// `None` is a no-op success; the caller doesn't have to gate on the
/// `Option`. Errors propagate `Err` from [`run_hook`] unchanged.
///
/// # Errors
///
/// Propagates any error from [`run_hook`] when `hook_opt` is `Some`.
pub fn run_hook_if_present(
  hook_opt: Option<&str>,
  label: &str,
) -> Result<(), Error> {
  if let Some(hook) = hook_opt {
    println!("Running the {label}-hook '{hook}'");
    let code = run_hook(hook_opt)?;
    tracing::debug!("{label}-hook script completed ok. RT={code}");
  }
  Ok(())
}

/// Verify that the program at the head of `hook_opt`'s command
/// string exists on disk and has the executable bit set. Only the
/// first whitespace-separated token is checked; trailing arguments
/// are ignored.
///
/// # Errors
///
/// - `hook_opt` is `None` or has no parseable program token.
/// - The program does not exist at the specified path.
/// - The file exists but is not executable.
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
