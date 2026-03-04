use crate::common::config::types::MantaConfiguration;
use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use config::Config;

/// Bundles the common configuration parameters that are threaded
/// through nearly every handler and command in the CLI.
///
/// By passing a single `&AppContext` instead of 8-11 individual
/// parameters, function signatures become manageable and the
/// `too_many_arguments` clippy warnings disappear.
pub struct AppContext<'a> {
  pub backend: &'a StaticBackendDispatcher,
  pub site_name: &'a str,
  pub shasta_base_url: &'a str,
  pub shasta_root_cert: &'a [u8],
  pub vault_base_url: Option<&'a String>,
  pub gitea_base_url: &'a str,
  pub k8s_api_url: Option<&'a String>,
  pub settings_hsm_group_name_opt: Option<&'a String>,
  pub kafka_audit_opt: Option<&'a Kafka>,
  pub settings: &'a Config,
  pub configuration: &'a MantaConfiguration,
}
