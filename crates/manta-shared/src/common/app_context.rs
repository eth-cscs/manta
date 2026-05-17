//! Context structs threaded through the call stack in CLI and server modes.
//!
//! The server uses [`InfraContext`] in its service layer (carries the
//! backend dispatcher, base URLs, TLS cert, vault/k8s/proxy URLs). The
//! CLI uses [`AppContext`] (carries a lightweight [`CliInfra`] with
//! just `site_name`, plus the CLI-only [`CliConfig`]). After Phase 7
//! the CLI never instantiates `StaticBackendDispatcher`, so the wider
//! `InfraContext` shape is server-only.

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

/// Lightweight infrastructure context for the CLI — only the
/// `site_name` the CLI needs to set the `X-Manta-Site` header on
/// outbound `MantaClient` requests.
#[derive(Debug)]
pub struct CliInfra<'a> {
  pub site_name: &'a str,
}

/// CLI-specific configuration that stays in the presentation layer:
/// user settings, HSM group filter, Kafka audit, etc.
#[derive(Debug)]
pub struct CliConfig<'a> {
  pub settings_hsm_group_name_opt: Option<&'a str>,
  pub kafka_audit_opt: Option<&'a Kafka>,
  pub settings: &'a Config,
  /// URL of the manta HTTP server this CLI talks to. Required.
  pub manta_server_url: &'a str,
}

/// Top-level CLI context — composes the lightweight [`CliInfra`] and
/// the [`CliConfig`]. Passed as `&AppContext` through CLI handlers
/// and commands.
#[derive(Debug)]
pub struct AppContext<'a> {
  pub infra: CliInfra<'a>,
  pub cli: CliConfig<'a>,
}
