//! Live `refresh` smoke test against one real manta-server.
//!
//! Dormant by default: it needs a reachable manta-server, so it only
//! runs when all three env vars are set. Without them it returns early
//! (a pass), so CI and contributors without a backend are unaffected.
//!
//! To run it:
//!
//! ```bash
//! MANTA_CACHE_IT_SERVER_URL="https://manta-server.example.ch:8443" \
//! MANTA_CACHE_IT_SITE="alps" \
//! MANTA_CACHE_IT_TOKEN="ey…" \
//!   cargo test -p manta-cache --test integration -- --nocapture
//! ```

use manta_cache::{SiteDescriptor, refresh};

#[tokio::test]
async fn refresh_against_live_server() {
  let (Ok(url), Ok(site), Ok(token)) = (
    std::env::var("MANTA_CACHE_IT_SERVER_URL"),
    std::env::var("MANTA_CACHE_IT_SITE"),
    std::env::var("MANTA_CACHE_IT_TOKEN"),
  ) else {
    eprintln!(
      "skipping integration test: set MANTA_CACHE_IT_SERVER_URL, \
       MANTA_CACHE_IT_SITE and MANTA_CACHE_IT_TOKEN to run it"
    );
    return;
  };

  let sites = [SiteDescriptor {
    name: site.clone(),
    manta_server_url: url,
    token,
  }];

  let index = refresh(&sites).await.expect("refresh failed");

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
