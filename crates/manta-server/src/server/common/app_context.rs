//! Server-side runtime infrastructure context.
//!
//! [`InfraContext`] is the bundle of per-site connection data passed
//! through the service layer for every request: backend dispatcher,
//! API base URLs, TLS cert, optional vault/k8s URLs, SOCKS proxy.
//! It depends on `StaticBackendDispatcher`, which is server-only —
//! the CLI never instantiates this.
//!
//! ## Lifetime
//!
//! `InfraContext<'a>` borrows everything from `ServerState`:
//! the backend dispatcher, URLs, root cert bytes, etc. live for the
//! whole server lifetime, but the borrow is taken anew per request.
//! Handlers obtain a context via
//! `state.infra_context(&site_name)` (returning
//! [`Result<InfraContext<'_>, Error>`]), then pass it by reference
//! into the service layer:
//!
//! ```ignore
//! let infra = state.infra_context(&site_name)?;
//! service::group::get_groups(&infra, &token, &params).await
//! ```
//!
//! The `_` lifetime is tied to the `state` borrow, so an
//! `InfraContext` cannot outlive the `Arc<ServerState>` that produced
//! it.
//!
//! ## Typical usage
//!
//! Service functions reach the backend through `infra.backend.*`
//! (calling the trait method belonging to the desired interface, e.g.
//! `infra.backend.get_bootparameters(...)`). When a function needs to
//! build a direct CSM HTTP client — e.g. for IMS customize jobs that
//! aren't routed through the dispatcher — it uses
//! `infra.shasta_base_url`, `infra.shasta_root_cert`, and
//! `infra.socks5_proxy` together. Vault- and k8s-dependent paths
//! gate on `infra.vault_base_url` / `infra.k8s_api_url` being
//! `Some`; when either is `None` the handler returns 501.

use crate::dispatcher::StaticBackendDispatcher;

/// Infrastructure context needed by the service layer: backend
/// dispatcher, API endpoints, and TLS certificates.
///
/// Constructed per-request by `ServerState::infra_context(site_name)`
/// from the matching `[sites.X]` block in `server.toml`. The borrows
/// live for the duration of the handler call.
#[derive(Debug)]
pub struct InfraContext<'a> {
  /// Backend client (CSM or OCHAMI) for this site.
  pub backend: &'a StaticBackendDispatcher,
  /// Name of the site this context belongs to, sourced from the
  /// `X-Manta-Site` header on the inbound request.
  pub site_name: &'a str,
  /// Base URL of the site's CSM / OpenCHAMI API
  /// (e.g. `https://api.alps.cscs.ch`).
  pub shasta_base_url: &'a str,
  /// DER- or PEM-encoded root CA bytes for verifying TLS against
  /// `shasta_base_url`.
  pub shasta_root_cert: &'a [u8],
  /// Optional per-site SOCKS5 proxy URL forwarded to every outbound
  /// HTTP request for this site's backend.
  pub socks5_proxy: Option<&'a str>,
  /// Optional Vault base URL; `None` makes Vault-dependent handlers
  /// return 501.
  pub vault_base_url: Option<&'a str>,
  /// Base URL of the site's Gitea VCS, used by SAT-file rendering
  /// and `run session` to resolve repository references.
  pub gitea_base_url: &'a str,
  /// Optional Kubernetes API URL; `None` makes k8s-dependent handlers
  /// (console, session-logs SSE) return 501.
  pub k8s_api_url: Option<&'a str>,
}
