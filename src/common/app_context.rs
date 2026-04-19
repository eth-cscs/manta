use crate::common::config::types::MantaConfiguration;
use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;

/// Infrastructure context needed by the service layer: backend
/// dispatcher, API endpoints, and TLS certificates.
pub struct InfraContext<'a> {
  pub backend: &'a StaticBackendDispatcher,
  pub site_name: &'a str,
  pub shasta_base_url: &'a str,
  pub shasta_root_cert: &'a [u8],
  pub vault_base_url: Option<&'a str>,
  pub gitea_base_url: &'a str,
  pub k8s_api_url: Option<&'a str>,
}

/// CLI-specific configuration that stays in the presentation layer:
/// user settings, HSM group filter, Kafka audit, etc.
pub struct CliConfig<'a> {
  pub settings_hsm_group_name_opt: Option<&'a str>,
  pub kafka_audit_opt: Option<&'a Kafka>,
  pub settings: &'a Config,
  pub configuration: &'a MantaConfiguration,
}

/// Top-level context that composes infrastructure and CLI config.
///
/// Passed as `&AppContext` through CLI handlers and commands.
/// Service-layer functions receive only `&InfraContext`.
pub struct AppContext<'a> {
  pub infra: InfraContext<'a>,
  pub cli: CliConfig<'a>,
}
