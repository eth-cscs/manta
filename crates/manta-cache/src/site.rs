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
///
/// The [`Debug`] impl redacts `token`, so descriptors are safe to log.
#[derive(Clone)]
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

// Manual impl so an accidental `{:?}` of a descriptor (or anything
// holding one) never writes the bearer token to a log.
impl std::fmt::Debug for SiteDescriptor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SiteDescriptor")
      .field("name", &self.name)
      .field("manta_server_url", &self.manta_server_url)
      .field("token", &"<redacted>")
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn debug_redacts_the_token() {
    let descriptor = SiteDescriptor {
      name: "alps".to_owned(),
      manta_server_url: "https://manta.example.ch:8443".to_owned(),
      token: "super-secret-bearer".to_owned(),
    };
    let rendered = format!("{descriptor:?}");
    assert!(!rendered.contains("super-secret-bearer"));
    assert!(rendered.contains("<redacted>"));
    assert!(rendered.contains("alps"));
  }
}
