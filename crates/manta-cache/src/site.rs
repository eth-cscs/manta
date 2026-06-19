//! Input descriptor identifying one site to refresh.

/// Everything [`crate::refresh`] needs to query one site.
///
/// A "site" here is one CSM/OpenCHAMI deployment as manta sees it. The
/// cache reaches it through a manta-server: it sends `name` as the
/// `X-Manta-Site` header and `token` as the bearer credential to the
/// group endpoints on `manta_server_url`.
///
/// Several sites can share one `manta_server_url` (a single
/// manta-server hosts every configured site); they are distinguished by
/// `name`.
#[derive(Debug, Clone)]
pub struct SiteDescriptor {
  /// Site identifier, sent verbatim as the `X-Manta-Site` header.
  pub name: String,
  /// Base URL of the manta-server hosting this site
  /// (e.g. `https://manta-server.example.ch:8443`). The `/api/v1`
  /// prefix is appended by the cache; a missing scheme defaults to
  /// `http://`.
  pub manta_server_url: String,
  /// Bearer token used for the group-listing calls. Per-site, scoped;
  /// where it comes from (service account vs per-user) is the caller's
  /// concern — see ROADMAP Stage 3.
  pub token: String,
}
