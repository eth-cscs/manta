//! `cache-server.toml` schema, path resolution, and validation.
//!
//! The file lives in the same platform config directory as the other
//! manta configs (`~/.config/manta/` on Linux), overridable with the
//! `MANTA_CACHE_SERVER_CONFIG` environment variable. The path helper is
//! a deliberate, tiny duplicate of `manta_shared::common::config`'s
//! `ProjectDirs` lookup: depending on `manta-shared` would drag
//! `manta-backend-dispatcher` (+ csm/ochami types) into this otherwise
//! standalone service for one function.

use std::collections::HashMap;
use std::path::PathBuf;

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

[sites.alps]
manta_server_url = "https://manta-server.example.com:8443"
# The site's Keycloak service-account token. Re-read on every refresh,
# so rotating the secret needs no restart. There is deliberately no
# inline form: this is a real credential and does not belong in a
# config file.
token_file = "/run/secrets/manta-cache/alps-token"
"#;

/// Top-level `cache-server.toml` schema.
///
/// `deny_unknown_fields` throughout the schema: a config file is
/// hand-edited and its keys are security-relevant, so a typo must be an
/// error rather than a silently ignored setting. Without it a mistyped
/// `api_token_fil` would start an unauthenticated server, and a
/// half-migrated `[sites.*] token` would leave a live Keycloak
/// credential sitting in the file while the service quietly used
/// something else.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheServerConfiguration {
  /// `tracing` filter string (e.g. `info`, `manta_cache=debug,info`).
  #[serde(default = "default_log")]
  pub log: String,
  /// Listener + refresh settings.
  #[serde(default)]
  pub server: ServerSettings,
  /// One entry per site the cache indexes, keyed by the site name sent
  /// as `X-Manta-Site`.
  #[serde(default)]
  pub sites: HashMap<String, SiteConfig>,
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
  /// Unlike the per-site `token_file`, this one is read **once at
  /// startup**: it guards the cache's own endpoints, so re-reading it
  /// per request would put a filesystem hit on the hot path, and
  /// rotating it means restarting anyway (in-flight clients hold the
  /// old value).
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

/// One `[sites.<name>]` block: where that site's manta-server lives and
/// the service-account credential used to refresh from it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
  /// Base URL of the manta-server hosting the site (scheme + host +
  /// port; `/api/v1` is appended by the cache).
  pub manta_server_url: String,
  /// Path to the file holding this site's Keycloak service-account
  /// bearer token.
  ///
  /// A path rather than an inline value, deliberately and without an
  /// inline alternative (ROADMAP Stage 5): a real credential must not
  /// live in a file that gets templated, backed up, or committed. It is
  /// also the one form every deployment target can produce — a
  /// Kubernetes projected `Secret`, a systemd `LoadCredential=`, or a
  /// bind mount all hand the process a path.
  ///
  /// Re-read on **every** refresh, so rotation needs no restart.
  pub token_file: PathBuf,
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
/// Also resolves `api_token_file` and warns about over-permissive
/// secret files, so everything downstream sees a config whose secrets
/// are already in hand.
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
      format!(
        "Config file '{}' is not valid: {e}{}",
        path.display(),
        migration_hint(&e.to_string())
      )
    })?;
  config.validate()?;
  config.resolve_secrets()?;
  config.warn_about_secret_file_permissions();
  Ok((config, path))
}

/// Extra explanation appended to a parse failure that looks like the
/// inline-`token` removal, empty otherwise.
///
/// Both halves of that migration surface as bare serde errors that name
/// a field but not the change behind it: dropping `token` without adding
/// `token_file` gives "missing field `token_file`", and keeping it gives
/// "unknown field `token`". Neither tells an operator staring at a file
/// that visibly contains a credential what actually happened, and this
/// was a breaking change — so the error carries the migration itself.
fn migration_hint(error: &str) -> &'static str {
  // Matched against the two exact serde phrasings rather than the bare
  // field names: `deny_unknown_fields` puts "expected `manta_server_url`
  // or `token_file`" into the message for *any* unknown key in a
  // `[sites.*]` table, so a looser test would append a paragraph about
  // deleting an inline `token` to someone who mistyped `timeout_secs`.
  if error.contains("unknown field `token`")
    || error.contains("missing field `token_file`")
  {
    "\n\nNote: `[sites.<name>] token` was removed — per-site credentials \
     are now file-based only. Write the site's Keycloak service-account \
     token to a file (mode 0600) and point `token_file` at it, then \
     delete the inline `token` line."
  } else {
    ""
  }
}

impl CacheServerConfiguration {
  /// Structural checks that TOML typing cannot express.
  pub fn validate(&self) -> Result<(), String> {
    if self.sites.is_empty() {
      return Err(
        "No [sites.<name>] entries configured — the cache would have \
         nothing to index. Add at least one site."
          .to_string(),
      );
    }
    if self.server.api_token.is_some() && self.server.api_token_file.is_some() {
      return Err(
        "[server] sets both `api_token` and `api_token_file`; configure \
         at most one."
          .to_string(),
      );
    }
    for (name, site) in &self.sites {
      if site.manta_server_url.trim().is_empty() {
        return Err(format!("[sites.{name}] `manta_server_url` is empty."));
      }
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

  /// Warn (never fail) about secret files readable beyond their owner.
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
  pub fn warn_about_secret_file_permissions(&self) {
    let mut paths: Vec<(String, &PathBuf)> = self
      .sites
      .iter()
      .map(|(name, site)| {
        (format!("[sites.{name}] token_file"), &site.token_file)
      })
      .collect();
    if let Some(path) = &self.server.api_token_file {
      paths.push(("[server] api_token_file".to_string(), path));
    }
    // HashMap iteration order would otherwise shuffle the warnings
    // between runs.
    paths.sort_by(|a, b| a.0.cmp(&b.0));

    for (label, path) in paths {
      if let Some(mode) = group_or_world_readable_mode(path) {
        eprintln!(
          "warning: {label} '{}' is readable beyond its owner (mode \
           {mode:04o}); tighten it to 0600 unless your secret store \
           sets the mode for you",
          path.display()
        );
      }
    }
  }
}

/// The file's permission bits when any of them grant group or world
/// access, `None` when the file is owner-only or cannot be stat'd (an
/// unreadable file is reported by the refresh, which produces a far
/// better message than a permission note would).
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
      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
      "#,
    );
    config.validate().expect("valid");
    assert_eq!(config.log, "info");
    assert_eq!(config.server.shutdown_grace_period_secs, 30);
    assert!(config.server.refresh_interval_secs.is_none());
  }

  #[test]
  fn a_half_migrated_inline_token_is_rejected_not_ignored() {
    // The dangerous upgrade path: the operator adds `token_file` but
    // leaves the old `token` behind. Silently ignoring it would park a
    // live Keycloak credential in the config file — exactly what moving
    // to file-based credentials was meant to prevent — while the service
    // quietly used something else.
    let err = toml::from_str::<CacheServerConfiguration>(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      token = "ey"
      token_file = "/run/secrets/alps"
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("unknown field `token`"), "{err}");
    assert!(migration_hint(&err).contains("was removed"), "{err}");
  }

  #[test]
  fn missing_credential_is_rejected() {
    let err = toml::from_str::<CacheServerConfiguration>(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("token_file"), "{err}");
    // A bare "missing field `token_file`" does not tell someone looking
    // at a file that still contains a credential what changed.
    assert!(migration_hint(&err).contains("service-account"), "{err}");
  }

  #[test]
  fn an_unrelated_unknown_site_key_gets_no_migration_paragraph() {
    // serde names `token_file` in the "expected …" half of every
    // unknown-key error inside a `[sites.*]` table, so a hint keyed on
    // the bare field name would lecture this operator about deleting an
    // inline `token` they never wrote.
    let err = toml::from_str::<CacheServerConfiguration>(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
      timeout_secs = 30
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("timeout_secs"), "{err}");
    assert!(err.contains("`token_file`"), "the trap this guards: {err}");
    assert!(migration_hint(&err).is_empty(), "{err}");
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

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
      "#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("api_token_fil"), "{err}");
    // Unrelated to the credential migration, so no hint is appended.
    assert!(migration_hint(&err).is_empty(), "{err}");
  }

  #[test]
  fn the_debug_impl_redacts_the_resolved_api_token() {
    let mut config = parse(
      r#"
      [server]
      api_token = "sekrit-inbound-bearer"

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
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

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
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

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
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

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
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

      [sites.alps]
      manta_server_url = "https://manta:8443"
      token_file = "/run/secrets/alps"
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
  fn no_sites_is_rejected() {
    let config = parse("log = \"info\"");
    let err = config.validate().unwrap_err();
    assert!(err.contains("No [sites"), "{err}");
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
