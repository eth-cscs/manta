//! CLI-side site pre-resolution through `manta-cache-server`
//! (manta-cache ROADMAP Stage 4).
//!
//! When a command targets an HSM group or a plain xname list but no
//! site was given (`--site` / `cli.toml` `site`), the CLI asks the
//! site-resolution cache named by `cli.toml`'s `cache_url` which site
//! owns the target, then proceeds exactly as if `--site <resolved>`
//! had been passed — same per-site token cache, same `X-Manta-Site`
//! header. The cache is an accelerator, not a dependency:
//!
//! - **Transport failures** (cache down, timeout, undecodable reply)
//!   print a warning and fall back to the usual lazy
//!   `AppContext::require_site` error.
//! - **Answers that make the command impossible** — unknown group,
//!   xname list split across sites, rejected `cache_api_token` — abort
//!   with a specific message; silently guessing a site could aim a
//!   destructive command at the wrong cluster.
//! - Targets the cache cannot know (hostlist expressions with
//!   brackets, NIDs) are skipped silently; those commands still need
//!   an explicit site.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use serde::Deserialize;

use crate::common::config::CliConfiguration;

/// TCP connect timeout for the cache call. Short on purpose: when the
/// cache is down, every site-less command pays this before degrading.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Whole-request timeout for the cache call — lookups are O(1) reads,
/// so anything slow means the cache is unhealthy and we degrade.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// What a command targets, as extracted from the parsed CLI matches.
#[derive(Debug, PartialEq, Eq)]
pub enum Target {
  /// A single HSM group label.
  Group(String),
  /// A plain, comma-separated xname list (no hostlist brackets, no
  /// NIDs — see [`parse_plain_xnames`]).
  Nodes(Vec<String>),
}

/// Walk the matched subcommand path and pull out the group label or
/// xname list for the commands the resolver understands.
///
/// The clap tree has no uniform target-argument id, so this is a
/// closed per-leaf table (extend it as commands grow resolution
/// support). `None` means "no resolvable target" and the command
/// follows the plain no-site path.
pub fn extract_target(root: &ArgMatches) -> Option<Target> {
  let mut path: Vec<&str> = Vec::new();
  let mut leaf = root;
  while let Some((name, sub)) = leaf.subcommand() {
    path.push(name);
    leaf = sub;
  }

  let group = |id: &str| {
    leaf
      .try_get_one::<String>(id)
      .ok()
      .flatten()
      .map(|label| Target::Group(label.clone()))
  };
  let nodes = |id: &str| {
    leaf
      .try_get_one::<String>(id)
      .ok()
      .flatten()
      .and_then(|raw| parse_plain_xnames(raw))
      .map(Target::Nodes)
  };

  match path.as_slice() {
    ["power", _, "group"] => group("GROUP_NAME"),
    ["power", _, "nodes"] => nodes("VALUE"),
    ["get", "nodes"] => nodes("VALUE"),
    ["get", "group-nodes"] => group("HSM_GROUP_NAME"),
    ["get", "sessions"] => group("group").or_else(|| nodes("xnames")),
    ["apply", "boot", "group"] => group("GROUP_NAME"),
    ["apply", "boot", "nodes"] => nodes("VALUE"),
    ["apply", "boot-parameters"] => nodes("hosts"),
    ["apply", "kernel-parameters"] => group("group").or_else(|| nodes("nodes")),
    ["console", "node"] => nodes("XNAME"),
    _ => None,
  }
}

/// Split a comma-separated node argument into xnames, or `None` when
/// any token is not a plain xname (hostlist brackets, NIDs, …) — the
/// cache indexes xnames only, so anything else must be resolved
/// server-side under an explicit site.
fn parse_plain_xnames(raw: &str) -> Option<Vec<String>> {
  let tokens: Vec<&str> = raw
    .split(',')
    .map(str::trim)
    .filter(|t| !t.is_empty())
    .collect();
  if tokens.is_empty() || !tokens.iter().all(|t| looks_like_xname(t)) {
    return None;
  }
  Some(tokens.into_iter().map(str::to_owned).collect())
}

/// Cheap shape check: `x` followed by a digit, no hostlist brackets.
/// Deliberately loose — a wrong-but-xname-shaped value simply comes
/// back from the cache as unknown.
fn looks_like_xname(token: &str) -> bool {
  let mut chars = token.chars();
  chars.next() == Some('x')
    && chars.next().is_some_and(|c| c.is_ascii_digit())
    && !token.contains(['[', ']'])
}

/// How a cache reply maps onto the resolution flow.
#[derive(Debug)]
enum Outcome {
  /// The cache named the site.
  Resolved(String),
  /// The cache is unusable (down, unhealthy, garbled) — warn with
  /// this message and fall back to the no-site path.
  Degrade(String),
}

/// `GET /lookup/group/{label}` reply body.
#[derive(Deserialize)]
struct GroupReply {
  site: String,
}

/// `GET /lookup/nodes` reply body.
#[derive(Deserialize)]
struct NodesReply {
  site: Option<String>,
  #[serde(default)]
  resolutions: BTreeMap<String, String>,
  #[serde(default)]
  unknown: Vec<String>,
}

/// Resolve the command's target site through the configured cache.
///
/// Returns `Ok(Some(site))` when the cache resolved it, `Ok(None)`
/// when resolution does not apply or degraded (no `cache_url`, no
/// resolvable target, cache unreachable — the latter after printing a
/// warning), and `Err` when the cache *answered* and the answer means
/// the command cannot proceed.
pub async fn resolve_via_cache(
  configuration: &CliConfiguration,
  matches: &ArgMatches,
) -> Result<Option<String>> {
  let Some(cache_url) = &configuration.cache_url else {
    return Ok(None);
  };
  let Some(target) = extract_target(matches) else {
    tracing::debug!("no cache-resolvable target in this command");
    return Ok(None);
  };

  let base = normalize_base(cache_url);
  let url = match &target {
    Target::Group(label) => format!("{base}/lookup/group/{label}"),
    Target::Nodes(xnames) => {
      format!("{base}/lookup/nodes?xnames={}", xnames.join(","))
    }
  };

  let client = reqwest::Client::builder()
    .connect_timeout(CONNECT_TIMEOUT)
    .timeout(REQUEST_TIMEOUT)
    .build()
    .context("failed to build the site-resolution HTTP client")?;
  let mut request = client.get(&url);
  if let Some(token) = &configuration.cache_api_token {
    request = request.bearer_auth(token);
  }

  let response = match request.send().await {
    Ok(response) => response,
    Err(e) => {
      degrade_warning(&format!("cache at {cache_url} unreachable: {e}"));
      return Ok(None);
    }
  };
  let status = response.status().as_u16();
  let body = response.text().await.unwrap_or_default();

  let outcome = match &target {
    Target::Group(label) => interpret_group_reply(status, &body, label)?,
    Target::Nodes(_) => interpret_nodes_reply(status, &body)?,
  };
  match outcome {
    Outcome::Resolved(site) => {
      eprintln!(
        "site '{site}' resolved via manta-cache ({})",
        target_desc(&target)
      );
      tracing::info!(site, url, "site resolved via manta-cache");
      Ok(Some(site))
    }
    Outcome::Degrade(reason) => {
      degrade_warning(&reason);
      Ok(None)
    }
  }
}

/// One-line human description of a target for the resolution notice.
fn target_desc(target: &Target) -> String {
  match target {
    Target::Group(label) => format!("group '{label}'"),
    Target::Nodes(xnames) => format!("nodes {}", xnames.join(",")),
  }
}

/// Shared warning line for every soft-degrade path.
fn degrade_warning(reason: &str) {
  eprintln!(
    "warning: site resolution degraded — {reason}; \
     pass --site <name> if the command fails without one"
  );
}

/// Map a `GET /lookup/group/{label}` reply. 404 and auth rejections
/// are hard errors; other non-success statuses degrade.
fn interpret_group_reply(
  status: u16,
  body: &str,
  label: &str,
) -> Result<Outcome> {
  match status {
    200 => match serde_json::from_str::<GroupReply>(body) {
      Ok(reply) => Ok(Outcome::Resolved(reply.site)),
      Err(e) => Ok(Outcome::Degrade(format!("undecodable cache reply: {e}"))),
    },
    404 => bail!(
      "the site-resolution cache knows no group '{label}'. \
       Pass --site <name> explicitly (or refresh the cache if the \
       group is new)."
    ),
    401 | 403 => bail!(auth_error(status)),
    other => Ok(Outcome::Degrade(format!("cache returned HTTP {other}"))),
  }
}

/// Map a `GET /lookup/nodes` reply. A split or partially-unknown list
/// is a hard error naming the per-xname resolutions.
fn interpret_nodes_reply(status: u16, body: &str) -> Result<Outcome> {
  match status {
    200 => match serde_json::from_str::<NodesReply>(body) {
      Ok(NodesReply {
        site: Some(site), ..
      }) => Ok(Outcome::Resolved(site)),
      Ok(reply) => {
        let mut detail: Vec<String> = reply
          .resolutions
          .iter()
          .map(|(xname, site)| format!("{xname} → {site}"))
          .collect();
        if !reply.unknown.is_empty() {
          detail.push(format!("unknown: {}", reply.unknown.join(", ")));
        }
        bail!(
          "the xnames do not resolve to a single site ({}). \
           Split the command per site or pass --site <name> explicitly.",
          detail.join("; ")
        );
      }
      Err(e) => Ok(Outcome::Degrade(format!("undecodable cache reply: {e}"))),
    },
    401 | 403 => bail!(auth_error(status)),
    other => Ok(Outcome::Degrade(format!("cache returned HTTP {other}"))),
  }
}

/// Message for a cache-side auth rejection — an actionable local
/// misconfiguration, so it aborts rather than degrades.
fn auth_error(status: u16) -> String {
  format!(
    "the site-resolution cache rejected the request (HTTP {status}). \
     Check `cache_api_token` in cli.toml against the cache's \
     [server] api_token."
  )
}

/// Normalise `cache_url` into `<scheme>://host[:port]/api/v1`, same
/// rules as `MantaClient` applies to `manta_server_url`: a missing
/// scheme defaults to `http://`.
fn normalize_base(url: &str) -> String {
  let with_scheme = if url.starts_with("http://") || url.starts_with("https://")
  {
    url.to_owned()
  } else {
    format!("http://{url}")
  };
  format!("{}/api/v1", with_scheme.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn target_for(argv: &[&str]) -> Option<Target> {
    let matches = crate::build::build_cli()
      .try_get_matches_from(argv)
      .expect("argv parses");
    extract_target(&matches)
  }

  #[test]
  fn extracts_group_targets() {
    assert_eq!(
      target_for(&["manta", "power", "off", "group", "zinal"]),
      Some(Target::Group("zinal".to_owned()))
    );
    assert_eq!(
      target_for(&["manta", "get", "group-nodes", "zinal"]),
      Some(Target::Group("zinal".to_owned()))
    );
    assert_eq!(
      target_for(&["manta", "apply", "boot", "group", "-i", "img-1", "zinal"]),
      Some(Target::Group("zinal".to_owned()))
    );
  }

  #[test]
  fn extracts_plain_xname_targets() {
    assert_eq!(
      target_for(&["manta", "get", "nodes", "x1000c0s0b0n0,x1000c0s0b0n1"]),
      Some(Target::Nodes(vec![
        "x1000c0s0b0n0".to_owned(),
        "x1000c0s0b0n1".to_owned()
      ]))
    );
    assert_eq!(
      target_for(&["manta", "console", "node", "x1000c0s0b0n0"]),
      Some(Target::Nodes(vec!["x1000c0s0b0n0".to_owned()]))
    );
  }

  #[test]
  fn hostlist_expressions_and_nids_are_not_targets() {
    // Bracket expression: needs server-side expansion.
    assert_eq!(
      target_for(&["manta", "get", "nodes", "x1000c0s[0-3]b0n0"]),
      None
    );
    // NID: the cache indexes xnames only.
    assert_eq!(target_for(&["manta", "get", "nodes", "nid001234"]), None);
  }

  #[test]
  fn commands_without_targets_are_skipped() {
    assert_eq!(target_for(&["manta", "get", "groups"]), None);
    assert_eq!(target_for(&["manta", "config", "show"]), None);
  }

  #[test]
  fn group_reply_resolves_404s_and_degrades() {
    let ok = interpret_group_reply(200, r#"{"site":"alps"}"#, "g").unwrap();
    assert!(matches!(ok, Outcome::Resolved(s) if s == "alps"));

    let err = interpret_group_reply(404, r#"{"error":"x"}"#, "g").unwrap_err();
    assert!(err.to_string().contains("knows no group 'g'"), "{err}");

    let err = interpret_group_reply(401, "", "g").unwrap_err();
    assert!(err.to_string().contains("cache_api_token"), "{err}");

    let deg = interpret_group_reply(500, "boom", "g").unwrap();
    assert!(matches!(deg, Outcome::Degrade(r) if r.contains("500")));
  }

  #[test]
  fn nodes_reply_split_is_a_hard_error_naming_resolutions() {
    let body = r#"{"site":null,
      "resolutions":{"x1":"alps","x2":"daint"},"unknown":["x9"]}"#;
    let err = interpret_nodes_reply(200, body).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("x1 → alps"), "{msg}");
    assert!(msg.contains("x2 → daint"), "{msg}");
    assert!(msg.contains("unknown: x9"), "{msg}");

    let ok = interpret_nodes_reply(
      200,
      r#"{"site":"alps","resolutions":{"x1":"alps"},"unknown":[]}"#,
    )
    .unwrap();
    assert!(matches!(ok, Outcome::Resolved(s) if s == "alps"));
  }

  #[test]
  fn normalize_base_defaults_scheme_and_appends_prefix() {
    assert_eq!(
      normalize_base("localhost:18081"),
      "http://localhost:18081/api/v1"
    );
    assert_eq!(
      normalize_base("https://cache.example.ch:8444/"),
      "https://cache.example.ch:8444/api/v1"
    );
  }
}
