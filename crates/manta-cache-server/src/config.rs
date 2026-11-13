//! `cache-server.toml` schema, path resolution, and validation.
//!
//! The file lives in the same platform config directory as the other
//! manta configs (`~/.config/manta/` on Linux), overridable with the
//! `MANTA_CACHE_SERVER_CONFIG` environment variable. The path helper
//! uses the same `ProjectDirs` triple as `manta_shared::common::config`,
//! so all manta configs share one directory.
//!
//! The service account the cache refreshes as is **not** configured
//! here: its credentials come from `MANTA_SA_CLIENT_ID` and
//! `MANTA_SA_CLIENT_SECRET` (see [`crate::auth`]). Only the Keycloak
//! issuer can be overridden, in `[auth]`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use manta_shared::common::oidc::DEFAULT_OIDC_ISSUER_URL;
use serde::Deserialize;

/// Sample config embedded in the "config file not found" error so an
/// operator can bootstrap without reading docs first.
const CONFIG_SAMPLE: &str = r#"log = "info"

[server]
# listen_address = "0.0.0.0"
# port = 8444                    # default: 8444 with TLS, 8081 without
# cert = "/path/to/cert.pem"     # omit both + allow_http for plain HTTP
# key  = "/path/to/key.pem"
# allow_http = false
# api_token = "shared-secret"    # require `Authorization: Bearer …` on /api/v1/*
# api_token_file = "/run/secrets/manta-cache/api-token"   # …or from a file
# refresh_interval_secs = 300    # omit = refresh at startup only
# shutdown_grace_period_secs = 30

# The cache refreshes as a CSCS Keycloak service account whose
# credentials come from the environment (MANTA_SA_CLIENT_ID,
# MANTA_SA_CLIENT_SECRET), never from this file.
# [auth]
# issuer_url = "https://auth.cscs.ch/auth/realms/cscs"   # the default

# The one manta-server the cache indexes and answers for, and which of
# its sites to index.
[manta_server]
url   = "https://manta-server.example.com:8443"
sites = ["alps"]
"#;

/// Top-level `cache-server.toml` schema.
///
/// `deny_unknown_fields` throughout the schema: a config file is
/// hand-edited and its keys are security-relevant, so a typo must be an
/// error rather than a silently ignored setting. Without it a mistyped
/// `api_token_fil` would start an unauthenticated server.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheServerConfiguration {
  /// `tracing` filter string (e.g. `info`, `manta_cache=debug,info`).
  #[serde(default = "default_log")]
  pub log: String,
  /// Listener + refresh settings.
  #[serde(default)]
  pub server: ServerSettings,
  /// Where the service account gets its tokens.
  #[serde(default)]
  pub auth: AuthSettings,
  /// The manta-server the cache indexes, and its sites to index.
  pub manta_server: MantaServerSettings,
}

/// `[server]` block.
///
/// The `Debug` impl is hand-written to redact `api_token`: after
/// [`CacheServerConfiguration::resolve_secrets`] this struct holds the
/// cache's inbound bearer secret in cleartext, and the same reasoning
/// that gave `manta_cache::SiteDescriptor` a redacting `Debug` applies
/// here — one stray `{:?}` should not put it in a log.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerSettings {
  /// Bind address; defaults to [`ServerSettings::DEFAULT_LISTEN_ADDRESS`].
  pub listen_address: Option<String>,
  /// TCP port; defaults per [`ServerSettings::default_port`].
  pub port: Option<u16>,
  /// PEM certificate path. Set together with `key` for TLS.
  pub cert: Option<String>,
  /// PEM private-key path.
  pub key: Option<String>,
  /// Allow plain HTTP when no cert/key is set (TLS terminates
  /// upstream). Defaults to fail-closed, like manta-server.
  #[serde(default)]
  pub allow_http: bool,
  /// Optional shared bearer token required on every `/api/v1/*` call
  /// (`/health` stays open for probes). Absent = unauthenticated
  /// lookups; rely on network controls instead.
  ///
  /// After [`load`] this holds the **effective** secret whichever form
  /// configured it: [`ServerSettings::api_token_file`] is read into this
  /// field by [`CacheServerConfiguration::resolve_secrets`], so request
  /// handling has a single place to look.
  pub api_token: Option<String>,
  /// Path to a file holding the `api_token`, for deployments that keep
  /// secrets out of config files. Mutually exclusive with `api_token`.
  ///
  /// Read **once at startup**: it guards the cache's own endpoints, so
  /// re-reading it per request would put a filesystem hit on the hot
  /// path, and rotating it means restarting anyway (in-flight clients
  /// hold the old value).
  pub api_token_file: Option<PathBuf>,
  /// Re-run the cross-site refresh every N seconds. Absent = refresh
  /// once at startup and serve that index until restart (Stage 4 adds
  /// on-demand refresh endpoints).
  pub refresh_interval_secs: Option<u64>,
  /// Drain window for graceful shutdown on SIGTERM / Ctrl+C.
  #[serde(default = "default_shutdown_grace_period_secs")]
  pub shutdown_grace_period_secs: u64,
}

// Redacts the resolved bearer secret; everything else is configuration
// worth seeing in a dump.
impl std::fmt::Debug for ServerSettings {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServerSettings")
      .field("listen_address", &self.listen_address)
      .field("port", &self.port)
      .field("cert", &self.cert)
      .field("key", &self.key)
      .field("allow_http", &self.allow_http)
      .field("api_token", &self.api_token.as_ref().map(|_| "<redacted>"))
      .field("api_token_file", &self.api_token_file)
      .field("refresh_interval_secs", &self.refresh_interval_secs)
      .field(
        "shutdown_grace_period_secs",
        &self.shutdown_grace_period_secs,
      )
      .finish()
  }
}

// Manual impl so an absent `[server]` table gets the same defaults as
// a present-but-sparse one — a derived Default would zero
// `shutdown_grace_period_secs` past the serde field default.
impl Default for ServerSettings {
  fn default() -> Self {
    Self {
      listen_address: None,
      port: None,
      cert: None,
      key: None,
      allow_http: false,
      api_token: None,
      api_token_file: None,
      refresh_interval_secs: None,
      shutdown_grace_period_secs: default_shutdown_grace_period_secs(),
    }
  }
}

impl ServerSettings {
  /// Default bind address: all interfaces (the service exists to be
  /// shared).
  pub const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0";

  /// Default port when neither config nor CLI flag supplies one.
  /// `8444` for HTTPS / `8081` for plain HTTP — one above
  /// manta-server's `8443`/`8080` so a colocated pair never collides.
  pub fn default_port(has_tls: bool) -> u16 {
    if has_tls { 8444 } else { 8081 }
  }
}

/// `[auth]` block: the Keycloak the service account belongs to. The
/// credentials themselves come from the environment (see
/// [`crate::auth`]).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthSettings {
  /// Issuer URL of the CSCS Keycloak realm; defaults to the production
  /// realm ([`DEFAULT_OIDC_ISSUER_URL`]), the same default as the CLI.
  pub issuer_url: Option<String>,
}

impl AuthSettings {
  /// The configured issuer, else the production CSCS realm.
  pub fn issuer_url(&self) -> &str {
    self
      .issuer_url
      .as_deref()
      .unwrap_or(DEFAULT_OIDC_ISSUER_URL)
  }
}

/// `[manta_server]` block: the one manta-server the cache indexes and
/// answers for.
///
/// One server, not one per site: the cache answers that server's
/// site-resolution queries (`[site_resolution]` in its `server.toml`),
/// so a site it does not host would be a useless, confusing answer.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MantaServerSettings {
  /// Base URL of the manta-server (scheme + host + port; `/v2` is
  /// appended by the cache).
  pub url: String,
  /// The sites to index, each sent as `X-Manta-Site`. They must be
  /// sites the server hosts (`[sites.<name>]` in its `server.toml`).
  pub sites: Vec<String>,
}

fn default_log() -> String {
  "info".to_string()
}

fn default_shutdown_grace_period_secs() -> u64 {
  30
}

/// Resolve the config file path: `MANTA_CACHE_SERVER_CONFIG` env var,
/// else `<platform config dir>/manta/cache-server.toml` (the same
/// directory as `server.toml` / `cli.toml`).
pub fn config_file_path() -> Result<PathBuf, String> {
  if let Ok(env_path) = std::env::var("MANTA_CACHE_SERVER_CONFIG") {
    return Ok(PathBuf::from(env_path));
  }
  // Same qualifier/organization/application triple as manta-shared's
  // `get_project_dirs`, so all manta configs share one directory.
  let dirs = directories::ProjectDirs::from("local", "cscs", "manta")
    .ok_or_else(|| {
      "Could not determine the config directory (home directory may \
       not be set); set MANTA_CACHE_SERVER_CONFIG instead."
        .to_string()
    })?;
  Ok(dirs.config_dir().join("cache-server.toml"))
}

/// Read and parse the config file, returning it with the path it was
/// loaded from (for the startup summary).
///
/// Also resolves `api_token_file` and warns when it is readable beyond
/// its owner, so everything downstream sees a config whose secret is
/// already in hand.
pub fn load() -> Result<(CacheServerConfiguration, PathBuf), String> {
  let path = config_file_path()?;
  let raw = std::fs::read_to_string(&path).map_err(|e| {
    format!(
      "Could not read config file '{}': {e}\n\n\
       Create it with content like:\n\n{CONFIG_SAMPLE}",
      path.display()
    )
  })?;
  let mut config: CacheServerConfiguration =
    toml::from_str(&raw).map_err(|e| {
      format!("Config file '{}' is not valid: {e}", path.display())
    })?;
  config.validate()?;
  config.resolve_secrets()?;
  config.warn_about_api_token_file_permissions();
  Ok((config, path))
}

impl CacheServerConfiguration {
  /// Structural checks that TOML typing cannot express.
  pub fn validate(&self) -> Result<(), String> {
    let manta_server = &self.manta_server;
    if manta_server.url.trim().is_empty() {
      return Err("[manta_server] `url` is empty.".to_string());
    }
    if manta_server.sites.is_empty() {
      return Err(
        "[manta_server] `sites` is empty — the cache would have nothing \
         to index. List at least one site."
          .to_string(),
      );
    }
    let mut seen = BTreeSet::new();
    for site in &manta_server.sites {
      if site.trim().is_empty() {
        return Err("[manta_server] `sites` has an empty name.".to_string());
      }
      if !seen.insert(site.as_str()) {
        return Err(format!("[manta_server] `sites` lists '{site}' twice."));
      }
    }
    if self.server.api_token.is_some() && self.server.api_token_file.is_some() {
      return Err(
        "[server] sets both `api_token` and `api_token_file`; configure \
         at most one."
          .to_string(),
      );
    }
    Ok(())
  }

  /// Read `api_token_file` into [`ServerSettings::api_token`], so the
  /// request path has one field to consult regardless of how the secret
  /// was configured.
  ///
  /// Fails rather than starting unauthenticated: an unreadable
  /// `api_token_file` means the operator asked for a guarded API and
  /// silently serving an open one would be the worst possible reading
  /// of that intent.
  pub fn resolve_secrets(&mut self) -> Result<(), String> {
    let Some(path) = &self.server.api_token_file else {
      return Ok(());
    };
    let token = std::fs::read_to_string(path)
      .map_err(|e| {
        format!(
          "[server] api_token_file '{}' could not be read: {e}",
          path.display()
        )
      })?
      .trim()
      .to_owned();
    if token.is_empty() {
      return Err(format!(
        "[server] api_token_file '{}' is empty; remove the setting to \
         serve unauthenticated, or write the shared secret into it.",
        path.display()
      ));
    }
    self.server.api_token = Some(token);
    Ok(())
  }

  /// Warn (never fail) when `api_token_file` is readable beyond its
  /// owner.
  ///
  /// Deliberately advisory: Kubernetes projected `Secret` volumes
  /// default to mode `0644`, so refusing to start would break the
  /// likeliest deployment target over a mode the operator frequently
  /// cannot change. Checked once at startup rather than per refresh —
  /// this is an operator-facing note, and repeating it on every tick
  /// would bury the log.
  ///
  /// Uses `eprintln!` rather than `tracing::warn!` **because it must**:
  /// [`load`] runs before `tracing_subscriber` is initialised in
  /// `main::run`, so a `tracing` call here would be swallowed with no
  /// test to catch the regression.
  pub fn warn_about_api_token_file_permissions(&self) {
    let Some(path) = &self.server.api_token_file else {
      return;
    };
    if let Some(mode) = group_or_world_readable_mode(path) {
      eprintln!(
        "warning: [server] api_token_file '{}' is readable beyond its \
         owner (mode {mode:04o}); tighten it to 0600 unless your secret \
         store sets the mode for you",
        path.display()
      );
    }
  }
}

/// The file's permission bits when any of them grant group or world
/// access, `None` when the file is owner-only or cannot be stat'd (an
/// unreadable file is reported by [`load`], which produces a far better
/// message than a permission note would).
#[cfg(unix)]
fn group_or_world_readable_mode(path: &std::path::Path) -> Option<u32> {
  use std::os::unix::fs::PermissionsExt;
  let mode = std::fs::metadata(path).ok()?.permissions().mode() & 0o777;
  (mode & 0o077 != 0).then_some(mode)
}

#[cfg(not(unix))]
fn group_or_world_readable_mode(_path: &std::path::Path) -> Option<u32> {
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(raw: &str) -> CacheServerConfiguration {
    toml::from_str(raw).expect("toml parses")
  }

  #[test]
  fn minimal_config_parses_and_validates() {
    let config = parse(
      r#"
      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    );
    config.validate().expect("valid");
    assert_eq!(config.log, "info");
    assert_eq!(config.server.shutdown_grace_period_secs, 30);
    assert!(config.server.refresh_interval_secs.is_none());
  }

  #[test]
  fn an_unknown_key_is_rejected_rather_than_ignored() {
    // Including the per-site tables of earlier versions of this PR.
    for raw in [
      "[manta_server]\nurl = \"https://manta:8443\"\nsites = [\"alps\"]\n\
       token_file = \"/run/secrets/alps\"\n",
      "[manta_server]\nurl = \"https://manta:8443\"\nsites = [\"alps\"]\n\
       [sites.alps]\nmanta_server_url = \"https://manta:8443\"\n",
    ] {
      let err = toml::from_str::<CacheServerConfiguration>(raw)
        .unwrap_err()
        .to_string();
      assert!(err.contains("unknown field"), "{err}");
    }
  }

  #[test]
  fn manta_server_is_required() {
    let err = toml::from_str::<CacheServerConfiguration>("log = \"info\"")
      .unwrap_err()
      .to_string();
    assert!(err.contains("manta_server"), "{err}");
  }

  #[test]
  fn the_site_list_is_validated() {
    for (sites, expected) in [
      ("[]", "is empty"),
      ("[\"alps\", \" \"]", "empty name"),
      ("[\"alps\", \"prealps\", \"alps\"]", "'alps' twice"),
    ] {
      let config = parse(&format!(
        "[manta_server]\nurl = \"https://manta:8443\"\nsites = {sites}\n"
      ));
      let err = config.validate().unwrap_err();
      assert!(err.contains(expected), "{sites}: {err}");
    }
    let config = parse("[manta_server]\nurl = \" \"\nsites = [\"alps\"]\n");
    assert!(config.validate().unwrap_err().contains("`url` is empty"));
  }

  #[test]
  fn the_issuer_defaults_to_the_production_realm_and_can_be_overridden() {
    let config = parse(
      r#"
      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    );
    assert_eq!(config.auth.issuer_url(), DEFAULT_OIDC_ISSUER_URL);

    let config = parse(
      r#"
      [auth]
      issuer_url = "https://auth-tds.cscs.ch/auth/realms/cscs"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    );
    assert_eq!(
      config.auth.issuer_url(),
      "https://auth-tds.cscs.ch/auth/realms/cscs"
    );
  }

  #[test]
  fn auth_rejects_credentials_in_the_file() {
    // The secret belongs in the environment; a `client_secret` here
    // must not be silently ignored.
    let err = toml::from_str::<CacheServerConfiguration>(
      r#"
      [auth]
      client_secret = "s3cret"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("client_secret"), "{err}");
  }

  #[test]
  fn a_mistyped_server_key_is_rejected_rather_than_ignored() {
    // Without deny_unknown_fields this typo starts a server with no
    // api_token at all: lookups unauthenticated, management 403, and
    // nothing said about the setting that did nothing.
    let err = toml::from_str::<CacheServerConfiguration>(
      r#"
      [server]
      api_token_fil = "/run/secrets/api"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("api_token_fil"), "{err}");
  }

  #[test]
  fn the_debug_impl_redacts_the_resolved_api_token() {
    let mut config = parse(
      r#"
      [server]
      api_token = "sekrit-inbound-bearer"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    );
    config.resolve_secrets().expect("nothing to resolve");
    let rendered = format!("{:?}", config.server);
    assert!(!rendered.contains("sekrit-inbound-bearer"), "{rendered}");
    assert!(rendered.contains("<redacted>"), "{rendered}");
  }

  #[test]
  fn both_api_token_forms_is_rejected() {
    let config = parse(
      r#"
      [server]
      api_token = "sekrit"
      api_token_file = "/run/secrets/api"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
    );
    let err = config.validate().unwrap_err();
    assert!(err.contains("at most one"), "{err}");
  }

  #[test]
  fn api_token_file_is_read_into_api_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("api-token");
    // Trailing newline is what an editor or `echo` leaves behind.
    std::fs::write(&path, "sekrit\n").unwrap();

    let mut config = parse(&format!(
      r#"
      [server]
      api_token_file = "{}"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
      path.display()
    ));
    config.resolve_secrets().expect("resolves");
    assert_eq!(config.server.api_token.as_deref(), Some("sekrit"));
  }

  #[test]
  fn an_unreadable_or_empty_api_token_file_fails_rather_than_serving_open() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("absent");
    let mut config = parse(&format!(
      r#"
      [server]
      api_token_file = "{}"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
      missing.display()
    ));
    let err = config.resolve_secrets().unwrap_err();
    assert!(err.contains("could not be read"), "{err}");

    let empty = dir.path().join("empty");
    std::fs::write(&empty, "  \n").unwrap();
    let mut config = parse(&format!(
      r#"
      [server]
      api_token_file = "{}"

      [manta_server]
      url = "https://manta:8443"
      sites = ["alps"]
      "#,
      empty.display()
    ));
    let err = config.resolve_secrets().unwrap_err();
    assert!(err.contains("is empty"), "{err}");
  }

  #[cfg(unix)]
  #[test]
  fn permission_check_flags_only_files_readable_beyond_the_owner() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let private = dir.path().join("private");
    std::fs::write(&private, "ey").unwrap();
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o600))
      .unwrap();
    assert_eq!(group_or_world_readable_mode(&private), None);

    // The mode a Kubernetes projected Secret defaults to: flagged, but
    // only ever as a warning.
    let shared = dir.path().join("shared");
    std::fs::write(&shared, "ey").unwrap();
    std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(0o644))
      .unwrap();
    assert_eq!(group_or_world_readable_mode(&shared), Some(0o644));

    // A path that does not exist is the refresh's problem to report.
    assert_eq!(group_or_world_readable_mode(&dir.path().join("nope")), None);
  }

  #[test]
  fn the_sample_config_in_the_not_found_error_is_itself_valid() {
    // The sample is what an operator copies when bootstrapping, so a
    // drifted one sends them straight into a parse error.
    let config: CacheServerConfiguration =
      toml::from_str(CONFIG_SAMPLE).expect("sample parses");
    config.validate().expect("sample validates");
  }

  #[test]
  fn default_ports_sit_next_to_manta_server() {
    assert_eq!(ServerSettings::default_port(true), 8444);
    assert_eq!(ServerSettings::default_port(false), 8081);
  }
}
