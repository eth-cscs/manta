//! CLI context struct threaded through `manta-cli`'s call stack.
//!
//! The server's analogous `InfraContext` (with backend dispatcher and
//! per-site URLs) lives in `manta-server::common::app_context` — it
//! depends on `StaticBackendDispatcher` which the CLI never touches.

use crate::common::kafka::Kafka;
use config::Config;

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
