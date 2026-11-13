//! Offline test of the real refresh path with the checked-in prealps
//! capture (`testdata/groups-prealps.json`, the verbatim payload of
//! `GET /v2/groups` against the CSCS prealps test site), served by a
//! mock manta-server: the response parsing and the fold into the
//! [`Index`](manta_cache::Index) are the library's own.
//!
//! The assertions mirror the fixture's "notable properties" documented
//! in ROADMAP.md: the site-umbrella group, empty groups, and a node
//! with overlapping group membership.

use manta_cache::{SiteDescriptor, TokenError, TokenSource, refresh};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Fixed;

impl TokenSource for Fixed {
  async fn token(&self) -> Result<String, TokenError> {
    Ok("tok".to_owned())
  }
}

#[tokio::test]
async fn prealps_fixture_builds_a_consistent_index() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/v2/groups"))
    .and(header("X-Manta-Site", "prealps"))
    .respond_with(
      ResponseTemplate::new(200)
        .insert_header("content-type", "application/json")
        .set_body_string(include_str!("../testdata/groups-prealps.json")),
    )
    .expect(1)
    .mount(&server)
    .await;

  let outcome = refresh(
    &[SiteDescriptor {
      name: "prealps".to_owned(),
      manta_server_url: server.uri(),
    }],
    &Fixed,
  )
  .await
  .expect("client build");
  assert!(outcome.is_complete(), "failures: {:?}", outcome.failures);
  let idx = outcome.index;

  // Single-site fixture: exactly one site, and all 17 groups known.
  assert_eq!(idx.sites().collect::<Vec<_>>(), vec!["prealps"]);
  assert_eq!(idx.groups().count(), 17);

  // Every label resolves to the site.
  for label in idx.groups().map(str::to_owned).collect::<Vec<_>>() {
    assert_eq!(idx.group_to_site(&label), Some("prealps"), "label {label}");
  }

  // Site-umbrella group: full node list of the site.
  let umbrella = idx.group_members("prealps").expect("umbrella group known");
  assert_eq!(umbrella.len(), 39);

  // Empty groups still resolve, with no members.
  for empty in ["cavel_arm", "k3s_agent"] {
    assert_eq!(idx.group_to_site(empty), Some("prealps"), "group {empty}");
    assert_eq!(idx.group_members(empty), Some(&[][..]), "group {empty}");
  }

  // Overlapping membership: one xname in several groups collapses onto
  // the same site, and appears in each group's member list.
  let xname = "x8000c1s5b1n0".to_owned();
  assert_eq!(idx.xname_to_site(&xname), Some("prealps"));
  for group in ["prealps", "rotondo", "cavel", "cavel_gh"] {
    assert!(
      idx
        .group_members(group)
        .expect("group known")
        .contains(&xname),
      "expected {xname} in {group}"
    );
  }
}
