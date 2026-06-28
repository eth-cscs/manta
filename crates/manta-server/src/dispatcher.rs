//! Runtime backend selector — wraps either a CSM or an OpenCHAMI
//! backend behind a single enum so the rest of the codebase is
//! backend-agnostic.
//!
//! [`StaticBackendDispatcher`] is constructed once per configured site
//! at startup (see [`crate::config::Site`]) and stored in the request
//! `ServerState`. Each HTTP request resolves a borrowed `InfraContext`
//! that holds a reference to the dispatcher for the requested site;
//! service code calls trait methods on that reference, and the trait
//! impls in [`crate::backend_dispatcher`] route the call to the active
//! variant.

use csm_rs::ShastaClient;
use manta_backend_dispatcher::error::Error;
use ochami_rs::backend_connector::Ochami;

/// Routes API calls to either a CSM or OCHAMI backend.
///
/// Every backend trait (e.g. `CfsTrait`, `GroupTrait`,
/// `BootParametersTrait`) is implemented for this enum under
/// [`crate::backend_dispatcher`]. The impls use the `dispatch!` macro
/// to forward the call to the wrapped client; both variants implement
/// the same trait surface so service code never branches on backend
/// kind.
///
/// Cloning is cheap — both inner clients are `Arc`-shaped internally.
/// Service helpers that need to move the dispatcher into a `'static`
/// spawned task call `InfraContext::backend_clone()` (see
/// [`crate::service::infra_backend`]).
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum StaticBackendDispatcher {
  /// HPE Cray System Management (CSM) backend, used by Alps-style
  /// deployments. Wraps a `csm-rs` HTTP client (`ShastaClient`).
  CSM(ShastaClient),
  /// OpenCHAMI backend, used by sites running the open-source CSM
  /// alternative. Wraps an `ochami-rs` HTTP client.
  OCHAMI(Ochami),
}

impl StaticBackendDispatcher {
  /// Returns `"csm"` or `"ochami"` for the currently-selected variant.
  /// Cheap, infallible — intended for use as a structured `tracing` field.
  pub fn backend_kind(&self) -> &'static str {
    match self {
      Self::CSM(_) => "csm",
      Self::OCHAMI(_) => "ochami",
    }
  }

  /// Create a new dispatcher for the given backend type.
  ///
  /// `backend_type` must be `"csm"` or `"ochami"` (matching
  /// [`crate::config::BackendTechnology::as_str`]); any other value
  /// returns [`Error::UnsupportedBackend`]. `root_cert` is the PEM
  /// bytes of the backend's root CA — used to verify TLS to
  /// `base_url`. `socks5_proxy` is an optional SOCKS5 URL applied to
  /// every outbound request.
  ///
  /// Called once per configured site during server startup.
  ///
  /// # Errors
  ///
  /// - [`Error::UnsupportedBackend`] when `backend_type` is not
  ///   `"csm"` or `"ochami"`.
  /// - Any error surfaced by `ShastaClient::new` (CSM variant) when
  ///   the supplied cert or proxy URL is unusable.
  pub fn new(
    backend_type: &str,
    base_url: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> Result<Self, Error> {
    match backend_type {
      "csm" => Ok(Self::CSM(ShastaClient::new(
        base_url,
        root_cert,
        socks5_proxy.map(str::to_string),
      )?)),
      "ochami" => {
        Ok(Self::OCHAMI(Ochami::new(base_url, root_cert, socks5_proxy)))
      }
      _ => Err(Error::UnsupportedBackend(format!(
        "Backend '{backend_type}' not supported"
      ))),
    }
  }
}
