//! Server-side runtime infrastructure context.
//!
//! `InfraContext` is the bundle of per-site connection data passed
//! through the service layer for every request: backend dispatcher,
//! API base URLs, TLS cert, optional vault/k8s URLs, SOCKS proxy.
//! It depends on `StaticBackendDispatcher`, which is server-only —
//! the CLI never instantiates this.
//!
//! `AppContext` is re-exported from `manta-shared` so handlers /
//! services that want the CLI shape can keep using
//! `crate::server::common::app_context::AppContext`.

#[allow(unused_imports)]
pub use manta_shared::common::app_context::AppContext;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Infrastructure context needed by the service layer: backend
/// dispatcher, API endpoints, and TLS certificates.
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
