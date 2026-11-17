//! Input descriptor identifying one site to refresh.

/// Everything [`crate::refresh`] needs to know about one site.
///
/// A "site" here is one CSM/OpenCHAMI deployment as manta sees it. The
/// cache reaches it through a manta-server: it sends `name` as the
/// `X-Manta-Site` header to the group endpoints on `manta_server_url`.
/// The bearer token is not per site: one [`crate::TokenSource`] serves
/// every site.
///
/// Several sites can share one `manta_server_url` (a single
/// manta-server hosts every configured site); they are distinguished by
/// `name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteDescriptor {
  /// Site identifier, sent verbatim as the `X-Manta-Site` header.
  pub name: String,
  /// Base URL of the manta-server hosting this site
  /// (e.g. `https://manta-server.example.ch:8443`). The `/v2`
  /// prefix is appended by the cache; a missing scheme defaults to
  /// `http://`.
  pub manta_server_url: String,
}
