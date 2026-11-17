//! Typed schema for `server.toml`.
//!
//! The untyped `config::Config` is loaded by
//! [`manta_shared::common::config::get_server_configuration`]; this
//! module owns the typed deserialisation target.
//!
//! The shape on disk is:
//!
//! ```toml
//! log = "info"
//!
//! [server]
//! listen_address = "0.0.0.0"
//! port = 8443
//! cert = "/etc/manta/tls/server.crt"
//! key = "/etc/manta/tls/server.key"
//! console_inactivity_timeout_secs = 1800
//!
//! [sites.alps]
//! backend = "csm"
//! shasta_base_url = "https://api.alps.cscs.ch"
//! root_ca_cert_file = "/etc/manta/alps-ca.pem"
//!
//! [sites.alps.token_exchange]
//! client_id = "cscs-token-exchanger"
//! client_secret = "…"
//! subject_issuer = "cscs"
//!
//! [sites.basodino]
//! backend = "ochami"
//! shasta_base_url = "https://basodino-mgmt.cscs.ch:8443"
//! root_ca_cert_file = "/etc/manta/basodino-ca.pem"
//!
//! [sites.basodino.token_exchange]
//! client_id = "…"
//! client_secret = "…"
//! scope = "smd.read bss.read smd.write bss.write"
//!
//! # Optional: resolve the site through manta-cache-server when a
//! # request carries no X-Manta-Site header.
//! [site_resolution]
//! url = "https://manta-cache.example.com:8444"
//! api_token_file = "/run/secrets/manta/cache-api-token"
//! ```
//!
//! [`ServerConfiguration`] is the top-level type; [`ServerSettings`]
//! and [`Site`] are its nested sections. There is no notion of an
//! "active" site — the server hosts every entry in `[sites.*]`
//! simultaneously and clients pick one per request via the
//! `X-Manta-Site` header.

use std::collections::HashMap;

use crate::server::audit::Auditor;
use manta_backend_dispatcher::{error::Error, types::K8sDetails};
use serde::{Deserialize, Serialize};

/// Which backend API this site speaks.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackendTechnology {
  /// HPE Cray System Management (CSM) backend.
  Csm,
  /// OpenCHAMI backend.
  Ochami,
}

impl BackendTechnology {
  /// Return the lowercase string expected by `StaticBackendDispatcher::new`.
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Csm => "csm",
      Self::Ochami => "ochami",
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
/// Connection details for a single ALPS site (CSM or OCHAMI instance).
///
/// The Vault URL used by handlers requiring vault (sat-file, session,
/// console, logs) is derived at startup from
/// `[sites.X.k8s.authentication.vault] base_url`. The vault secret path
/// is derived from a hard-coded prefix and the site name. Neither is
/// configured here.
pub struct Site {
  /// Which backend implementation this site uses (`csm` or `ochami`).
  pub backend: BackendTechnology,
  /// Base URL of the backend API (e.g. `https://api.alps.cscs.ch`).
  pub shasta_base_url: String,
  /// Optional Kubernetes connection details, required by handlers
  /// that stream CFS session logs or attach to consoles.
  pub k8s: Option<K8sDetails>,
  /// Path (absolute or relative to the config dir) of the backend's
  /// root CA certificate, used to verify TLS to `shasta_base_url`.
  pub root_ca_cert_file: String,
  /// How to exchange the caller's CSCS Keycloak token for a token this
  /// site accepts. Required: the server refuses to start without it.
  pub token_exchange: Option<TokenExchangeConfig>,
}

/// `[sites.X.token_exchange]`: client credentials for exchanging the
/// caller's CSCS Keycloak token for a token the site accepts.
///
/// - `csm` sites use an RFC 8693 token exchange at the site's Keycloak
///   (`shasta` realm), at a URL derived from `shasta_base_url`
///   (`<base without a trailing /apis>/keycloak`). They need
///   `subject_issuer`.
/// - `ochami` sites use an RFC 7523 JWT bearer grant at the site's
///   Hydra, at a URL derived from `shasta_base_url`
///   (`<base>/hydra/oauth2/token`). They need `scope`.
///
/// [`TokenExchangeConfig::csm`] and [`TokenExchangeConfig::ochami`]
/// check that the backend's fields are present.
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenExchangeConfig {
  /// Client allowed to exchange tokens, e.g. `cscs-token-exchanger`.
  pub client_id: String,
  /// Secret of the confidential `client_id`.
  pub client_secret: String,
  /// `csm` only: alias of the CSCS identity provider in the `shasta`
  /// realm, e.g. `cscs`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub subject_issuer: Option<String>,
  /// `ochami` only: space-separated scopes to request.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub scope: Option<String>,
}

impl std::fmt::Debug for TokenExchangeConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TokenExchangeConfig")
      .field("client_id", &self.client_id)
      .field("client_secret", &"<redacted>")
      .field("subject_issuer", &self.subject_issuer)
      .field("scope", &self.scope)
      .finish()
  }
}

impl TokenExchangeConfig {
  /// The credentials as `csm-rs` takes them.
  ///
  /// # Errors
  ///
  /// [`Error::MissingField`] when `subject_issuer` is not set.
  pub fn csm(&self) -> Result<csm_rs::backend_connector::TokenExchange, Error> {
    Ok(csm_rs::backend_connector::TokenExchange {
      client_id: self.client_id.clone(),
      client_secret: self.client_secret.clone(),
      subject_issuer: required(&self.subject_issuer, "subject_issuer", "csm")?,
    })
  }

  /// The credentials as `ochami-rs` takes them.
  ///
  /// # Errors
  ///
  /// [`Error::MissingField`] when `scope` is not set.
  pub fn ochami(
    &self,
  ) -> Result<ochami_rs::authentication::TokenExchange, Error> {
    Ok(ochami_rs::authentication::TokenExchange {
      client_id: self.client_id.clone(),
      client_secret: self.client_secret.clone(),
      scope: required(&self.scope, "scope", "ochami")?,
    })
  }
}

fn required(
  value: &Option<String>,
  field: &str,
  backend: &str,
) -> Result<String, Error> {
  value.clone().ok_or_else(|| {
    Error::MissingField(format!(
      "token_exchange.{field} is required for backend = \"{backend}\""
    ))
  })
}

/// Server-only settings — TLS, listen address, console behaviour. Lives
/// under `[server]` in `server.toml`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerSettings {
  /// TCP listen address (e.g. "0.0.0.0"). When omitted from config
  /// **and** no `--listen-address` flag is supplied, the server falls
  /// back to `"0.0.0.0"`.
  #[serde(default)]
  pub listen_address: Option<String>,
  /// TCP port. When omitted from config **and** no `--port` flag is
  /// supplied, the effective default depends on whether TLS is
  /// configured: `8443` if both `cert` and `key` are present (HTTPS),
  /// otherwise `8080` (plain HTTP). See
  /// [`ServerSettings::default_port`].
  #[serde(default)]
  pub port: Option<u16>,
  /// Path to the TLS certificate (PEM).
  pub cert: Option<String>,
  /// Path to the TLS private key (PEM).
  pub key: Option<String>,
  /// How long a node-console WebSocket stays open without activity
  /// before the server tears it down.
  pub console_inactivity_timeout_secs: u64,
  /// Global request timeout applied to every HTTP route, in seconds.
  /// When this elapses the server returns `408 REQUEST_TIMEOUT`. All
  /// long-running work (e.g. power transitions) now runs CLI-side,
  /// so no endpoint needs more than the default.
  #[serde(default = "default_request_timeout_secs")]
  pub request_timeout_secs: u64,
  /// Grace period (seconds) `axum_server` waits for in-flight
  /// requests to finish after SIGTERM / Ctrl+C before force-aborting.
  /// Matches the standard k8s `terminationGracePeriodSeconds` default
  /// (30 s); pods that hit this without finishing get SIGKILL'd by
  /// the kubelet.
  #[serde(default = "default_shutdown_grace_period_secs")]
  pub shutdown_grace_period_secs: u64,
  /// Filesystem root that confines `POST /migrate/{backup,restore}`
  /// file access. When set, every `destination` / `bos_file` /
  /// `cfs_file` / `hsm_file` / `ims_file` / `image_dir` path in the
  /// request is canonicalised and rejected unless it resolves under
  /// this directory. When unset (default), the migrate endpoints
  /// return `BadRequest` even for admin callers — the operator must
  /// explicitly opt in to server-side filesystem writes.
  #[serde(default)]
  pub migrate_backup_root: Option<String>,
  /// Opt in to plain-HTTP listen mode. Default `false`: when neither
  /// `cert` nor `key` is configured the server refuses to start, so
  /// bearer tokens can't accidentally land on the wire in cleartext.
  /// Set to `true` only when TLS terminates upstream (reverse proxy
  /// or sidecar); otherwise leave it off and configure both `cert`
  /// and `key`.
  #[serde(default)]
  pub allow_http: bool,
}

impl ServerSettings {
  /// Effective default listen address when neither config nor CLI flag
  /// supplies one: bind on all interfaces.
  pub const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0";

  /// Effective default port when neither config nor CLI flag supplies
  /// one. `8443` for the HTTPS path (cert + key both present), `8080`
  /// for plain HTTP — the latter is the typical dev / sidecar setup
  /// where TLS is terminated upstream.
  pub fn default_port(has_tls: bool) -> u16 {
    if has_tls { 8443 } else { 8080 }
  }
}

/// Default global request timeout — 600s (10 min). Bumped from 300s
/// after `GET /cache/configuration` (and other cross-resource fan-out
/// endpoints that hit CFS + BSS + IMS concurrently) started 408'ing
/// against busy sites where any single upstream fetch can stall on
/// the CSM Envoy reset window. 10 min still bounds truly hung
/// requests while letting heavy-but-healthy ones through. Override
/// via `request_timeout_secs` in `server.toml` if your deployment
/// needs a different ceiling.
fn default_shutdown_grace_period_secs() -> u64 {
  30
}

fn default_request_timeout_secs() -> u64 {
  600
}

/// Top-level configuration for the `manta-server` binary.
///
/// Persisted as TOML under `~/.config/manta/server.toml` and loaded
/// once at startup. Has no notion of an "active" site — the server
/// hosts every entry in [`Self::sites`] simultaneously and clients
/// select per-request via the `X-Manta-Site` header.
///
/// See the module-level docs for the on-disk layout.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
  /// `EnvFilter` directive for the tracing subscriber.
  pub log: String,
  /// Network / TLS / console / timeout knobs for the HTTPS server.
  pub server: ServerSettings,
  /// Per-site backend connection details, keyed by site name. The
  /// `X-Manta-Site` header on each request picks which one to route to.
  pub sites: HashMap<String, Site>,
  /// Optional Kafka destination for the audit trail. When `None`,
  /// audit events go to the log (`manta::audit` target) only.
  pub auditor: Option<Auditor>,
  /// Optional `manta-cache-server` used to pick the site for requests
  /// that carry no `X-Manta-Site` header. When `None`, such requests
  /// are rejected with 400 exactly as before.
  pub site_resolution: Option<SiteResolutionConfig>,
}

/// `[site_resolution]`: where to ask which site owns a group or an
/// xname list when a request names no site.
///
/// `Debug` is hand-written to redact `api_token`.
#[derive(Serialize, Deserialize, Clone)]
pub struct SiteResolutionConfig {
  /// Base URL of `manta-cache-server` (e.g.
  /// `https://manta-cache.example.com:8444`). A missing scheme
  /// defaults to `http://`; `/api/v1` is appended.
  pub url: String,
  /// The cache's `[server] api_token`, sent as `Authorization: Bearer
  /// …` on every lookup. Omit when the cache serves lookups without a
  /// token. Inline like the other secrets in this file; at most one of
  /// this and [`Self::api_token_file`].
  pub api_token: Option<String>,
  /// Path of a file holding the cache's `[server] api_token`, for a
  /// deployment that keeps it out of this file. Read once at startup.
  pub api_token_file: Option<String>,
  /// PEM CA certificate to trust for the cache's TLS, in addition to
  /// the system roots (absolute, or relative to the config dir).
  pub root_ca_cert_file: Option<String>,
}

impl std::fmt::Debug for SiteResolutionConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SiteResolutionConfig")
      .field("url", &self.url)
      .field("api_token", &self.api_token.as_ref().map(|_| "<redacted>"))
      .field("api_token_file", &self.api_token_file)
      .field("root_ca_cert_file", &self.root_ca_cert_file)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn site_resolution_debug_redacts_the_inline_token() {
    let config = SiteResolutionConfig {
      url: "http://localhost:8081".to_owned(),
      api_token: Some("s3cret".to_owned()),
      api_token_file: None,
      root_ca_cert_file: None,
    };
    let rendered = format!("{config:?}");
    assert!(!rendered.contains("s3cret"), "{rendered}");
    assert!(rendered.contains("<redacted>"), "{rendered}");
  }

  #[test]
  fn site_deserialize_missing_backend_fails() {
    let bad_toml = r#"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"
      # missing backend
    "#;
    let result = toml::from_str::<Site>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn backend_technology_as_str() {
    assert_eq!(BackendTechnology::Csm.as_str(), "csm");
    assert_eq!(BackendTechnology::Ochami.as_str(), "ochami");
  }

  #[test]
  fn backend_technology_roundtrip_toml() {
    // Verify TOML serializes as lowercase "csm" / "ochami"
    #[derive(Serialize, Deserialize)]
    struct Wrapper {
      backend: BackendTechnology,
    }
    let w = Wrapper {
      backend: BackendTechnology::Csm,
    };
    let s = toml::to_string(&w).unwrap();
    assert!(s.contains("\"csm\"") || s.contains("csm"));
    let parsed: Wrapper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.backend, BackendTechnology::Csm);
  }

  fn make_minimal_site() -> Site {
    Site {
      backend: BackendTechnology::Csm,
      shasta_base_url: "https://api.example.com".to_string(),
      k8s: None,
      root_ca_cert_file: "cert.pem".to_string(),
      token_exchange: Some(TokenExchangeConfig {
        client_id: "cscs-token-exchanger".to_string(),
        client_secret: "s3cret".to_string(),
        subject_issuer: Some("cscs".to_string()),
        scope: None,
      }),
    }
  }

  /// `examples/server.toml`, with its commented-out sites enabled, is a
  /// config the server starts with: every site has the token exchange
  /// its backend needs (`run_server` refuses a site without one, and
  /// [`TokenExchangeConfig::csm`] / [`TokenExchangeConfig::ochami`] check
  /// the backend's fields).
  #[test]
  fn the_example_server_toml_is_a_valid_config() {
    let path =
      concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/server.toml");
    let example = std::fs::read_to_string(path).unwrap();
    // Each commented-out site: a `# [sites.<name>]` line and the comment
    // lines right after it, with the `# ` taken off. (Not the commented
    // alternatives inside an enabled site, such as `alps`' `Native`.)
    let is_site_header = |line: &str| {
      line
        .strip_prefix("# [sites.")
        .and_then(|rest| rest.strip_suffix(']'))
        .is_some_and(|name| !name.contains('.'))
    };
    let mut enabled = String::new();
    let mut in_site = false;
    for line in example.lines() {
      if is_site_header(line) {
        in_site = true;
      } else if !line.starts_with('#') {
        in_site = false;
      }
      if in_site {
        let toml_line = line.strip_prefix('#').unwrap_or(line);
        enabled.push_str(toml_line.strip_prefix(' ').unwrap_or(toml_line));
        enabled.push('\n');
      }
    }
    let config: ServerConfiguration =
      toml::from_str(&format!("{example}\n{enabled}")).unwrap();

    assert!(
      config.sites.contains_key("ochami-dev"),
      "{:?}",
      config.sites.keys()
    );
    for (name, site) in &config.sites {
      let exchange = site
        .token_exchange
        .as_ref()
        .unwrap_or_else(|| panic!("[sites.{name}] has no token_exchange"));
      match site.backend {
        BackendTechnology::Csm => assert!(exchange.csm().is_ok(), "{name}"),
        BackendTechnology::Ochami => {
          assert!(exchange.ochami().is_ok(), "{name}");
        }
      }
    }
  }

  #[test]
  fn site_resolution_parses_and_is_optional() {
    let cfg: ServerConfiguration = toml::from_str(
      r#"
      log = "info"
      [server]
      console_inactivity_timeout_secs = 1800
      [sites]
      [site_resolution]
      url = "https://manta-cache.example.com:8444"
      api_token_file = "/run/secrets/cache-token"
    "#,
    )
    .unwrap();
    let r = cfg.site_resolution.expect("[site_resolution] parsed");
    assert_eq!(r.url, "https://manta-cache.example.com:8444");
    assert_eq!(
      r.api_token_file.as_deref(),
      Some("/run/secrets/cache-token")
    );
    assert!(r.root_ca_cert_file.is_none());

    let cfg: ServerConfiguration = toml::from_str(
      r#"
      log = "info"
      [server]
      console_inactivity_timeout_secs = 1800
      [sites]
    "#,
    )
    .unwrap();
    assert!(cfg.site_resolution.is_none());
  }

  #[test]
  fn token_exchange_may_be_absent_in_toml() {
    let site: Site = toml::from_str(
      r#"
      backend = "ochami"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"
    "#,
    )
    .unwrap();
    assert!(site.token_exchange.is_none());
  }

  #[test]
  fn token_exchange_parses_and_redacts_the_secret() {
    let site: Site = toml::from_str(
      r#"
      backend = "csm"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"

      [token_exchange]
      client_id = "cscs-token-exchanger"
      client_secret = "s3cret"
      subject_issuer = "cscs"
    "#,
    )
    .unwrap();
    let te = site.token_exchange.expect("token_exchange parsed");
    assert_eq!(te.client_id, "cscs-token-exchanger");
    assert_eq!(te.client_secret, "s3cret");
    assert_eq!(te.subject_issuer.as_deref(), Some("cscs"));
    assert_eq!(te.csm().unwrap().subject_issuer, "cscs");
    assert!(!format!("{te:?}").contains("s3cret"));
  }

  #[test]
  fn ochami_token_exchange_parses() {
    let site: Site = toml::from_str(
      r#"
      backend = "ochami"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"

      [token_exchange]
      client_id = "client"
      client_secret = "s3cret"
      scope = "smd.read bss.read"
    "#,
    )
    .unwrap();
    let te = site
      .token_exchange
      .expect("token_exchange parsed")
      .ochami()
      .unwrap();
    assert_eq!(te.scope, "smd.read bss.read");
  }

  #[test]
  fn each_backend_names_the_token_exchange_field_it_misses() {
    let te = TokenExchangeConfig {
      client_id: "client".to_string(),
      client_secret: "s3cret".to_string(),
      subject_issuer: None,
      scope: None,
    };
    let csm_err = te.csm().unwrap_err().to_string();
    assert!(csm_err.contains("subject_issuer"), "{csm_err}");
    let ochami_err = te.ochami().unwrap_err().to_string();
    assert!(ochami_err.contains("scope"), "{ochami_err}");
  }

  #[test]
  fn server_configuration_roundtrip_toml_minimal() {
    let mut sites = HashMap::new();
    sites.insert("alps".to_string(), make_minimal_site());
    let cfg = ServerConfiguration {
      log: "info".to_string(),
      server: ServerSettings {
        listen_address: Some("0.0.0.0".to_string()),
        port: Some(8443),
        cert: Some("/etc/manta/tls/server.crt".to_string()),
        key: Some("/etc/manta/tls/server.key".to_string()),
        console_inactivity_timeout_secs: 1800,
        request_timeout_secs: 300,
        shutdown_grace_period_secs: 30,
        migrate_backup_root: None,
        allow_http: false,
      },
      sites,
      auditor: None,
      site_resolution: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: ServerConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.server.port, Some(8443));
    assert_eq!(parsed.server.listen_address.as_deref(), Some("0.0.0.0"));
    assert_eq!(parsed.server.console_inactivity_timeout_secs, 1800);
    assert_eq!(parsed.server.request_timeout_secs, 300);
    assert_eq!(
      parsed.server.cert.as_deref(),
      Some("/etc/manta/tls/server.crt")
    );
  }

  /// Default port helper: 8443 when TLS is configured, 8080
  /// otherwise. Used by `manta-server::main` when no `port` is set
  /// in config or on the CLI.
  #[test]
  fn server_settings_default_port_depends_on_tls() {
    assert_eq!(ServerSettings::default_port(true), 8443);
    assert_eq!(ServerSettings::default_port(false), 8080);
  }

  /// power_timeout_secs is gone — confirm the surrounding
  /// timeout-related fields still default correctly when the only
  /// remaining knob is absent.
  #[test]
  fn server_settings_request_timeout_secs_defaults_to_600() {
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert_eq!(parsed.request_timeout_secs, 600);
  }

  /// `[server]` block with neither `listen_address` nor `port`
  /// supplied — both fields deserialise as `None`, leaving the
  /// effective values to be filled in at startup time. Confirms the
  /// schema-level back-compat for the new defaults.
  #[test]
  fn server_settings_listen_address_and_port_default_to_none() {
    let toml_str = r#"
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert!(parsed.listen_address.is_none());
    assert!(parsed.port.is_none());
  }

  /// Existing server.toml files that pre-date the request_timeout
  /// field must keep working — the field falls back to its default.
  #[test]
  fn server_settings_request_timeout_field_defaults_when_omitted() {
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert_eq!(parsed.request_timeout_secs, 600);
  }

  #[test]
  fn server_configuration_deserialize_missing_server_section_fails() {
    let bad_toml = r#"
      log = "info"
      [sites]
    "#;
    let result = toml::from_str::<ServerConfiguration>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn server_settings_optional_tls_paths() {
    // TLS cert/key are optional in the schema — flags can supply them
    // at runtime when the config omits them.
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert!(parsed.cert.is_none());
    assert!(parsed.key.is_none());
  }
}
