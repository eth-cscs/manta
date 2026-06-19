//! Error type for the site-resolution cache.
//!
//! `manta-cache` is its own crate and is not bound by the layer-
//! partitioned error rule that splits `anyhow` / `MantaError` /
//! `BackendError` across the binaries (that CI grep covers only
//! specific `manta-server` / `manta-shared` paths). It owns a small
//! crate-local error enum instead, named `CacheError` as the ROADMAP
//! specifies.

use thiserror::Error;

/// Failure modes of [`crate::refresh`].
///
/// Each network failure carries the offending site name so a caller
/// fanning out over many sites can tell which one broke.
#[derive(Debug, Error)]
pub enum CacheError {
  /// The shared `reqwest::Client` could not be constructed (e.g. TLS
  /// backend initialisation failed). Not tied to any one site.
  #[error("failed to build HTTP client: {0}")]
  ClientBuild(#[source] reqwest::Error),

  /// An outbound request to a site's manta-server failed at the
  /// transport layer, or its JSON body could not be decoded.
  #[error("request to site '{site}' failed: {source}")]
  Request {
    /// Site (`X-Manta-Site`) whose request failed.
    site: String,
    /// Underlying `reqwest` transport or decode error.
    #[source]
    source: reqwest::Error,
  },

  /// A site's manta-server responded with a non-success HTTP status.
  #[error("site '{site}' returned HTTP {status}")]
  Status {
    /// Site (`X-Manta-Site`) whose request was rejected.
    site: String,
    /// The HTTP status code returned by manta-server.
    status: u16,
  },
}
