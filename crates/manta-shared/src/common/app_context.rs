//! Context structs threaded through the call stack in CLI and server modes.
//!
//! The server uses [`InfraContext`] in its service layer (carries the
//! backend dispatcher, base URLs, TLS cert, vault/k8s/proxy URLs).
//! The CLI uses [`AppContext`] — a flat struct with the `site_name`
//! for the `X-Manta-Site` header, the manta-server URL, plus the
//! CLI-only configuration (settings, HSM filter, Kafka audit).

use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;

/// Infrastructure context needed by the service layer: backend
/// dispatcher, API endpoints, and TLS certificates. **Server-only**
/// after Phase 7.
#[derive(Debug)]
pub struct InfraContext<'a> {
  pub backend: &'a StaticBackendDispatcher,
  pub site_name: &'a str,
  pub shasta_base_url: &'a str,
  pub shasta_root_cert: &'a [u8],
  pub socks5_proxy: Option<&'a str>,
  pub vault_base_url: Option<&'a str>,
  pub gitea_base_url: &'a str,
  pub k8s_api_url: Option<&'a str>,
}

/// Top-level CLI context, passed as `&AppContext` through CLI
/// handlers and commands.
#[derive(Debug)]
pub struct AppContext<'a> {
  /// Site name used to set the `X-Manta-Site` header on outbound
  /// `MantaClient` requests.
  pub site_name: &'a str,
  /// URL of the manta HTTP server this CLI talks to. Required.
  pub manta_server_url: &'a str,
  pub settings_hsm_group_name_opt: Option<&'a str>,
  pub kafka_audit_opt: Option<&'a Kafka>,
  pub settings: &'a Config,
}
