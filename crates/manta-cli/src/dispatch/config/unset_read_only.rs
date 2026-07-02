//! `manta config unset read-only` — removes the `read_only` key from `cli.toml`.
//!
//! Inverse of [`super::set_read_only`]: backend-mutating subcommands
//! become callable again on the next invocation.

use anyhow::{Context, Error};

use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};
use toml_edit::DocumentMut;

/// Disable read-only mode by removing the `read_only` key from
/// `cli.toml`.
///
/// # Errors
///
/// Returns an error if the config file cannot be read or written, or
/// the renderer fails.
pub async fn exec() -> Result<(), Error> {
  let (path, mut doc) =
    read_config_toml().context("Could not read CLI configuration file")?;
  unset_read_only_in_doc(&mut doc);
  write_config_toml(&path, &doc)
    .context("Could not write CLI configuration file")?;

  action_result::print("Read-only mode disabled.", None)?;

  Ok(())
}

/// Pure helper: remove `read_only` from `doc` if present.
/// Idempotent — a no-op when the key is already absent.
fn unset_read_only_in_doc(doc: &mut DocumentMut) {
  doc.remove("read_only");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn removes_read_only_when_present() {
    let mut doc: DocumentMut =
      "site = \"alps\"\nread_only = true\n".parse().unwrap();
    unset_read_only_in_doc(&mut doc);
    let out = doc.to_string();
    assert!(!out.contains("read_only"), "key should be gone: {out}");
    assert!(
      out.contains("site = \"alps\""),
      "site should survive: {out}"
    );
  }

  #[test]
  fn is_idempotent_when_already_absent() {
    let mut doc: DocumentMut = "site = \"alps\"\n".parse().unwrap();
    unset_read_only_in_doc(&mut doc);
    assert!(doc.to_string().contains("site = \"alps\""));
  }

  #[test]
  fn removes_read_only_when_false() {
    let mut doc: DocumentMut = "read_only = false\n".parse().unwrap();
    unset_read_only_in_doc(&mut doc);
    assert!(!doc.to_string().contains("read_only"));
  }
}
