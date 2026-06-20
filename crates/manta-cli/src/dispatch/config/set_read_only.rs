//! `manta config set read-only` — writes `read_only = true` to `cli.toml`.

use anyhow::{Context, Error};
use manta_shared::common::config::{read_config_toml, write_config_toml};
use toml_edit::DocumentMut;

pub async fn exec() -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()
    .context("Could not read CLI configuration file")?;
  set_read_only_in_doc(&mut doc);
  write_config_toml(&path, &doc)
    .context("Could not write CLI configuration file")?;
  println!(
    "Read-only mode enabled. Backend-mutating commands will be refused. \
     Disable with `manta config unset read-only`."
  );
  Ok(())
}

/// Pure helper: set `read_only = true` on `doc`. Idempotent.
fn set_read_only_in_doc(doc: &mut DocumentMut) {
  doc["read_only"] = toml_edit::value(true);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sets_read_only_to_true_on_empty_doc() {
    let mut doc: DocumentMut = "".parse().unwrap();
    set_read_only_in_doc(&mut doc);
    assert!(doc.to_string().contains("read_only = true"));
  }

  #[test]
  fn sets_read_only_to_true_alongside_other_fields() {
    let mut doc: DocumentMut =
      "site = \"alps\"\nmanta_server_url = \"https://x:8443\"\n"
        .parse()
        .unwrap();
    set_read_only_in_doc(&mut doc);
    let out = doc.to_string();
    assert!(out.contains("read_only = true"), "got: {out}");
    assert!(out.contains("site = \"alps\""), "site should survive: {out}");
  }

  #[test]
  fn is_idempotent_when_already_true() {
    let mut doc: DocumentMut = "read_only = true\n".parse().unwrap();
    set_read_only_in_doc(&mut doc);
    let out = doc.to_string();
    assert_eq!(
      out.matches("read_only").count(),
      1,
      "must not duplicate the key: {out}"
    );
    assert!(out.contains("read_only = true"));
  }

  #[test]
  fn overwrites_false_with_true() {
    let mut doc: DocumentMut = "read_only = false\n".parse().unwrap();
    set_read_only_in_doc(&mut doc);
    assert!(doc.to_string().contains("read_only = true"));
  }
}
