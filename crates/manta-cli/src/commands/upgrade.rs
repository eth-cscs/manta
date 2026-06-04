//! Implements the `manta upgrade` command.
//!
//! Fetches the highest `manta-cli-v*` tag from
//! <https://github.com/eth-cscs/manta/releases>, compares against the
//! currently running binary's version (`env!("CARGO_PKG_VERSION")`),
//! and replaces the binary with the platform-appropriate tarball.
//!
//! The repo ships three crate tags per workspace bump
//! (`manta-cli-v*`, `manta-server-v*`, `manta-shared-v*`), so this
//! command filters strictly to `manta-cli-v*` rather than relying on
//! GitHub's "latest release" pointer.
//!
//! Archive format is `.tar.xz` (cargo-dist default for Unix targets),
//! containing `manta-cli-{target}/manta` plus docs/completions. We
//! extract just the binary to a tempfile in the same directory as the
//! current exe (so a same-filesystem rename works), then
//! `fs::rename` it into place. The running process keeps executing
//! via the kept-open inode; subsequent `manta` invocations pick up
//! the new binary.

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use semver::Version;
use serde_json::{Value, json};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::common::confirm::confirm;
use crate::output::action_result;

const REPO_OWNER: &str = "eth-cscs";
const REPO_NAME: &str = "manta";
const TAG_PREFIX: &str = "manta-cli-v";

/// Result of the version check; serialised under `data` when
/// `--output json` is requested.
#[derive(serde::Serialize)]
struct VersionInfo<'a> {
  current: &'a str,
  latest: String,
  target: &'a str,
  asset_url: String,
  up_to_date: bool,
}

pub fn exec(
  check_only: bool,
  dry_run: bool,
  assume_yes: bool,
  output_opt: Option<&str>,
) -> Result<()> {
  let current_str = env!("CARGO_PKG_VERSION");
  let current = Version::parse(current_str)
    .with_context(|| format!("could not parse current version '{current_str}'"))?;

  let target = self_update::get_target();
  ensure_supported_target(target)?;

  let latest = fetch_latest_cli_version()?;

  let asset_name = format!("manta-cli-{target}.tar.xz");
  let asset_url = format!(
    "https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/download/\
     {TAG_PREFIX}{latest}/{asset_name}"
  );

  let up_to_date = latest <= current;
  let info = VersionInfo {
    current: current_str,
    latest: latest.to_string(),
    target,
    asset_url: asset_url.clone(),
    up_to_date,
  };

  if up_to_date {
    action_result::print_with_data(
      &format!("Already up to date (v{current_str})."),
      &info,
      output_opt,
    )?;
    return Ok(());
  }

  let message = format!(
    "A newer manta is available: v{current_str} → v{latest}"
  );
  action_result::print_with_data(&message, &info, output_opt)?;

  if check_only || dry_run {
    return Ok(());
  }

  // Warn (don't block) when the binary path looks brew-managed; brew
  // will simply overwrite our replacement on its next `brew upgrade`.
  let exe_path = env::current_exe().context("could not locate the running manta binary")?;
  if looks_like_homebrew_path(&exe_path) {
    eprintln!(
      "warning: this `manta` binary appears to be Homebrew-managed \
       ({}); consider `brew upgrade manta-cli` instead. Continuing anyway.",
      exe_path.display()
    );
  }

  if !confirm(
    &format!("Replace {} with v{latest}?", exe_path.display()),
    assume_yes,
  ) {
    bail!("upgrade cancelled by user");
  }

  let new_bin = download_and_extract(&asset_url, target, &exe_path)?;
  fs::rename(&new_bin, &exe_path).with_context(|| {
    format!(
      "failed to replace {} with the new binary at {}",
      exe_path.display(),
      new_bin.display()
    )
  })?;

  action_result::print_with_data(
    &format!("Replaced {} with v{latest}.", exe_path.display()),
    &json!({"installed_version": latest.to_string()}),
    output_opt,
  )?;

  Ok(())
}

/// Return an error if the rust target triple isn't one we publish
/// release tarballs for.
fn ensure_supported_target(target: &str) -> Result<()> {
  const SUPPORTED: &[&str] = &[
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
  ];
  if !SUPPORTED.contains(&target) {
    bail!(
      "no published release for target '{target}'. \
       Supported: {:?}. Build from source or open an issue.",
      SUPPORTED
    );
  }
  Ok(())
}

/// Hit the GitHub releases API, filter to `manta-cli-v*` tags, and
/// return the highest semver.
fn fetch_latest_cli_version() -> Result<Version> {
  let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases");
  // Need a `User-Agent` — GitHub rejects API requests without one.
  let ua = concat!("manta-cli/", env!("CARGO_PKG_VERSION"));
  let resp: Vec<Value> = reqwest::blocking::Client::new()
    .get(&url)
    .header(reqwest::header::USER_AGENT, ua)
    .header(reqwest::header::ACCEPT, "application/vnd.github+json")
    .send()
    .with_context(|| format!("failed to GET {url}"))?
    .error_for_status()
    .with_context(|| format!("GitHub returned an error for {url}"))?
    .json()
    .context("failed to parse GitHub releases response as JSON")?;

  let mut versions: Vec<Version> = resp
    .iter()
    .filter_map(|r| r.get("tag_name").and_then(Value::as_str))
    .filter_map(|tag| tag.strip_prefix(TAG_PREFIX))
    .filter_map(|ver| Version::parse(ver).ok())
    .collect();

  versions.sort();
  versions
    .pop()
    .ok_or_else(|| anyhow!("no '{TAG_PREFIX}*' releases found at {url}"))
}

/// Download the tarball, extract the `manta` binary into a tempfile
/// in the same directory as `exe_path` (so we can rename across the
/// same filesystem), set it executable, and return its path.
fn download_and_extract(
  asset_url: &str,
  target: &str,
  exe_path: &Path,
) -> Result<PathBuf> {
  eprintln!("Downloading {asset_url}");
  let bytes = reqwest::blocking::Client::new()
    .get(asset_url)
    .header(
      reqwest::header::USER_AGENT,
      concat!("manta-cli/", env!("CARGO_PKG_VERSION")),
    )
    .send()
    .with_context(|| format!("failed to GET {asset_url}"))?
    .error_for_status()
    .with_context(|| format!("download failed for {asset_url}"))?
    .bytes()
    .context("failed to read tarball bytes")?;

  let mut archive = Archive::new(XzDecoder::new(bytes.as_ref()));
  let inner_path = format!("manta-cli-{target}/manta");

  for entry in archive.entries().context("failed to iterate tar entries")? {
    let mut entry = entry.context("failed to read a tar entry")?;
    let path = entry.path().context("tar entry has no path")?.to_path_buf();
    if path.to_str() == Some(&inner_path) {
      let parent = exe_path.parent().ok_or_else(|| {
        anyhow!("current exe path {} has no parent directory", exe_path.display())
      })?;
      let tmp = parent.join(format!(
        ".manta.upgrade.{}.tmp",
        std::process::id()
      ));
      let mut out = File::create(&tmp)
        .with_context(|| format!("failed to create temp file {}", tmp.display()))?;
      let mut buf = Vec::new();
      entry
        .read_to_end(&mut buf)
        .context("failed to read tar entry bytes")?;
      out
        .write_all(&buf)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
      out.sync_all().ok();
      drop(out);
      set_executable(&tmp)?;
      return Ok(tmp);
    }
  }

  bail!("tarball did not contain expected file '{inner_path}'")
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
  use std::os::unix::fs::PermissionsExt;
  let mut perms = fs::metadata(path)
    .with_context(|| format!("failed to stat {}", path.display()))?
    .permissions();
  perms.set_mode(0o755);
  fs::set_permissions(path, perms)
    .with_context(|| format!("failed to chmod 755 {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
  Ok(())
}

fn looks_like_homebrew_path(p: &Path) -> bool {
  let s = p.to_string_lossy();
  s.contains("/Cellar/")
    || s.starts_with("/opt/homebrew/")
    || s.starts_with("/usr/local/bin/")
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn brew_path_detection_matches_arm_cellar() {
    let p = PathBuf::from(
      "/opt/homebrew/Cellar/manta-cli/2.0.0-beta.27/bin/manta",
    );
    assert!(looks_like_homebrew_path(&p));
  }

  #[test]
  fn brew_path_detection_matches_intel_cellar() {
    let p = PathBuf::from(
      "/usr/local/Cellar/manta-cli/2.0.0-beta.27/bin/manta",
    );
    assert!(looks_like_homebrew_path(&p));
  }

  #[test]
  fn brew_path_detection_matches_opt_homebrew_bin() {
    let p = PathBuf::from("/opt/homebrew/bin/manta");
    assert!(looks_like_homebrew_path(&p));
  }

  #[test]
  fn brew_path_detection_does_not_fire_for_cargo_home() {
    let p = PathBuf::from("/Users/alice/.cargo/bin/manta");
    assert!(!looks_like_homebrew_path(&p));
  }

  #[test]
  fn ensure_supported_target_accepts_known_targets() {
    assert!(ensure_supported_target("x86_64-apple-darwin").is_ok());
    assert!(ensure_supported_target("aarch64-apple-darwin").is_ok());
    assert!(ensure_supported_target("x86_64-unknown-linux-gnu").is_ok());
    assert!(ensure_supported_target("aarch64-unknown-linux-gnu").is_ok());
  }

  #[test]
  fn ensure_supported_target_rejects_unknown_target() {
    assert!(ensure_supported_target("riscv64-unknown-linux-gnu").is_err());
  }
}
