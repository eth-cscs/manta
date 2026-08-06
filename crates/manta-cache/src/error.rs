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
  #[error("site '{site}' returned HTTP {status}: {body}")]
  Status {
    /// Site (`X-Manta-Site`) whose request was rejected.
    site: String,
    /// The HTTP status code returned by manta-server.
    status: u16,
    /// Bounded snippet of the response body — manta-server returns a
    /// small JSON error object whose message is the actual diagnosis
    /// (e.g. distinguishing an expired token from an unknown site).
    /// `<empty body>` when the response had none.
    body: String,
  },
}

impl CacheError {
  /// The site this failure belongs to, or `None` for
  /// [`CacheError::ClientBuild`] — the one variant that precedes the
  /// fan-out and so blames no single site.
  ///
  /// Lets a caller fanning out over many sites attribute each failure
  /// structurally instead of parsing the [`Display`](std::fmt::Display)
  /// text.
  pub fn site(&self) -> Option<&str> {
    match self {
      CacheError::ClientBuild(_) => None,
      CacheError::Request { site, .. } | CacheError::Status { site, .. } => {
        Some(site)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // `ClientBuild`'s `None` arm is not covered: `reqwest::Error` has no
  // public constructor, so the variant cannot be built in a test. It is
  // exhaustively matched above, so a new variant still breaks the build.
  #[test]
  fn site_names_the_failing_site() {
    let status = CacheError::Status {
      site: "alps".to_owned(),
      status: 401,
      body: "{\"error\":\"expired token\"}".to_owned(),
    };
    assert_eq!(status.site(), Some("alps"));
  }
}
