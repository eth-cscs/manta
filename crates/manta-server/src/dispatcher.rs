//! Runtime backend selector — wraps either a CSM or an OpenCHAMI backend
//! behind a single enum so the rest of the codebase is backend-agnostic.

use csm_rs::ShastaClient;
use manta_backend_dispatcher::error::Error;
use ochami_rs::backend_connector::Ochami;

/// Routes API calls to either a CSM or OCHAMI backend.
///
/// All backend-specific trait methods are dispatched via
/// the `dispatch!` macro defined in the
/// [`crate::backend_dispatcher`] module.
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
  /// `backend_type` must be `"csm"` or `"ochami"`;
  /// any other value returns an error.
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
