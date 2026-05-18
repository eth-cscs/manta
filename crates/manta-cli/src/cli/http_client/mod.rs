//! Thin HTTP client for forwarding CLI calls to the manta server.
//!
//! Every CLI command goes through this client; the server resolves CA
//! certs, base URLs, and credentials internally — the CLI only sends
//! `X-Manta-Site` + `Authorization: Bearer <token>`.
//!
//! The endpoint methods are split per-resource across sub-modules
//! (mirroring `crates/manta-server/src/server/handlers/`) so each
//! file covers one slice of the HTTP API. All methods are still
//! `pub fn`s on the single `MantaClient` struct — the split is
//! purely organisational: external callers (`client.get_sessions(...)`,
//! `client.add_node(...)`, …) don't change.

use anyhow::{Context, bail};
use serde::de::DeserializeOwned;

mod auth;
mod boot_parameters;
mod clusters;
mod configurations;
mod console;
mod ephemeral_env;
mod groups;
mod hardware;
mod hw_cluster;
mod images;
mod kernel_parameters;
mod migrate;
mod nodes;
mod power;
mod redfish_endpoints;
mod sat_file;
mod sessions;
mod templates;

/// HTTP client that forwards CLI requests to a manta server.
#[derive(Debug)]
pub struct MantaClient {
  client: reqwest::Client,
  base_url: String,
  site_name: String,
}

impl MantaClient {
  /// Build a client pointing at `server_url` for the given `site_name`.
  ///
  /// If `server_url` has no scheme, `http://` is prepended. This lets users
  /// write `manta_server_url = "localhost:8080"` in their config without
  /// triggering a "URL scheme is not allowed" error from reqwest.
  pub fn new(server_url: &str, site_name: &str) -> anyhow::Result<Self> {
    let normalized = if server_url.starts_with("http://")
      || server_url.starts_with("https://")
    {
      server_url.to_owned()
    } else {
      format!("http://{}", server_url)
    };

    let client = reqwest::Client::builder()
      .build()
      .context("Failed to build HTTP client")?;
    Ok(Self {
      client,
      base_url: format!("{}/api/v1", normalized.trim_end_matches('/')),
      site_name: site_name.to_owned(),
    })
  }

  // ── shared helpers (visible to sub-modules) ───────────────────────────────
  //
  // These are `pub(super)` so the resource sub-modules can call them. They
  // are the only places that touch `reqwest` directly; sub-module methods
  // build a URL fragment + query/body and delegate here.

  pub(super) async fn parse_json<T: DeserializeOwned>(
    resp: reqwest::Response,
  ) -> anyhow::Result<T> {
    if resp.status().is_success() {
      resp
        .json::<T>()
        .await
        .context("Failed to parse response JSON")
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {}: {}", status, body)
    }
  }

  pub(super) async fn parse_no_content(
    resp: reqwest::Response,
  ) -> anyhow::Result<()> {
    if resp.status().is_success() {
      Ok(())
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {}: {}", status, body)
    }
  }

  pub(super) async fn get_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .get(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP GET failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn post_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .post(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP POST failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn put_no_content(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .put(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP PUT failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content(
    &self,
    token: &str,
    path: &str,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content_with_query(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_no_content_with_body(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  pub(super) async fn delete_json_with_body<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  pub(super) async fn delete_json_with_query<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  // Accessors used by sub-modules that build URLs / set headers directly
  // (SSE streaming, WebSocket consoles).
  pub(super) fn http_client(&self) -> &reqwest::Client {
    &self.client
  }
  pub(super) fn base_url(&self) -> &str {
    &self.base_url
  }
  pub(super) fn site_name(&self) -> &str {
    &self.site_name
  }
}

/// Chainable builder for the `&[(&str, String)]` query-pairs slice
/// that `MantaClient::get_json` expects. Each `.opt()` / `.vec()` /
/// `.flag()` / `.pair()` call mirrors one of the patterns the older
/// hand-written query blocks used; absent values are skipped.
#[derive(Default)]
pub(super) struct QueryBuilder {
  pairs: Vec<(&'static str, String)>,
}

impl QueryBuilder {
  pub(super) fn new() -> Self {
    Self::default()
  }

  /// Push `(name, value.clone())` only when `value` is `Some`.
  pub(super) fn opt(
    mut self,
    name: &'static str,
    value: &Option<String>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.clone()));
    }
    self
  }

  /// Push `(name, value.to_string())` only when `value` is `Some`.
  /// For numeric `Option<T>` where `T: ToString`.
  pub(super) fn opt_display<T: ToString>(
    mut self,
    name: &'static str,
    value: &Option<T>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.to_string()));
    }
    self
  }

  /// Push `(name, items.join(","))` only when `items` is non-empty.
  pub(super) fn vec(mut self, name: &'static str, items: &[String]) -> Self {
    if !items.is_empty() {
      self.pairs.push((name, items.join(",")));
    }
    self
  }

  /// Push `(name, "true")` only when `value` is `true`.
  pub(super) fn flag(mut self, name: &'static str, value: bool) -> Self {
    if value {
      self.pairs.push((name, "true".to_string()));
    }
    self
  }

  /// Push `(name, value)` unconditionally.
  pub(super) fn pair(mut self, name: &'static str, value: String) -> Self {
    self.pairs.push((name, value));
    self
  }

  /// Consume into the slice-shaped form `get_json` accepts.
  pub(super) fn build(self) -> Vec<(&'static str, String)> {
    self.pairs
  }
}

/// Convert an `http://` or `https://` base URL to the corresponding `ws://` / `wss://` URL.
pub(super) fn ws_base_url(http_url: &str) -> String {
  if let Some(rest) = http_url.strip_prefix("https://") {
    format!("wss://{}", rest)
  } else if let Some(rest) = http_url.strip_prefix("http://") {
    format!("ws://{}", rest)
  } else {
    http_url.to_owned()
  }
}
