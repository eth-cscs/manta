//! Live `refresh` smoke test against one real manta-server.
//!
//! Dormant by default: it needs a reachable manta-server and a CSCS
//! service account, so it only runs when the server URL, the site and
//! the service account's two variables are set. Without them it returns
//! early (a pass), so CI and contributors without a backend are
//! unaffected.
//!
//! To run it:
//!
//! ```bash
//! MANTA_CACHE_IT_SERVER_URL="https://manta-server.example.ch:8443" \
//! MANTA_CACHE_IT_SITE="alps" \
//! MANTA_SA_CLIENT_ID="…" MANTA_SA_CLIENT_SECRET="…" \
//!   cargo test -p manta-cache --test integration -- --nocapture
//! ```
//!
//! `MANTA_CACHE_IT_ISSUER_URL` overrides the CSCS Keycloak issuer
//! (default: the production realm).

use manta_cache::{SiteDescriptor, TokenError, TokenSource, refresh};
use manta_shared::common::oidc::{DEFAULT_OIDC_ISSUER_URL, ServiceAccount};

/// A new token per request from the service account.
struct Account {
  account: ServiceAccount,
  issuer_url: String,
}

impl TokenSource for Account {
  async fn token(&self) -> Result<String, TokenError> {
    Ok(self.account.token(&self.issuer_url).await?)
  }
}

#[tokio::test]
async fn refresh_against_live_server() {
  let (Ok(url), Ok(site), Ok(Some(account))) = (
    std::env::var("MANTA_CACHE_IT_SERVER_URL"),
    std::env::var("MANTA_CACHE_IT_SITE"),
    ServiceAccount::from_env(),
  ) else {
    eprintln!(
      "skipping integration test: set MANTA_CACHE_IT_SERVER_URL, \
       MANTA_CACHE_IT_SITE, MANTA_SA_CLIENT_ID and MANTA_SA_CLIENT_SECRET \
       to run it"
    );
    return;
  };
  let tokens = Account {
    account,
    issuer_url: std::env::var("MANTA_CACHE_IT_ISSUER_URL")
      .unwrap_or_else(|_| DEFAULT_OIDC_ISSUER_URL.to_owned()),
  };

  let sites = [SiteDescriptor {
    name: site.clone(),
    manta_server_url: url,
  }];

  let outcome = refresh(&sites, &tokens).await.expect("refresh failed");
  assert!(
    outcome.is_complete(),
    "site refresh failures: {:?}",
    outcome.failures
  );
  let index = outcome.index;

  // The configured site must be present in the resolved index.
  assert!(
    index.sites().any(|s| s == site),
    "expected site '{site}' in the index, got {:?}",
    index.sites().collect::<Vec<_>>()
  );

  // Self-consistency: every group resolves to a site, and every member
  // xname resolves back to the same site as its group.
  for group in index.groups().map(str::to_owned).collect::<Vec<_>>() {
    let group_site = index
      .group_to_site(&group)
      .unwrap_or_else(|| panic!("group '{group}' has no site"));

    for xname in index.group_members(&group).unwrap_or(&[]) {
      assert_eq!(
        index.xname_to_site(xname),
        Some(group_site),
        "xname '{xname}' (group '{group}') resolved to a different site"
      );
    }
  }
}
