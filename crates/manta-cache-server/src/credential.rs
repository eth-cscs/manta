//! Advisory facts about a bearer credential, read from its own claims.
//!
//! The cache refreshes each site with a Keycloak service-account token
//! (ROADMAP Stage 5). Nothing here *enforces* that — a token is an
//! opaque bearer string as far as the refresh is concerned, and pointing
//! `token_file` at a personal token still works, which is what keeps the
//! [LIVE-TEST](../../manta-cache/LIVE-TEST.md) demo flow usable. What
//! this module does is make the difference **observable**: which
//! principal a site refreshes as, when its credential expires, and
//! whether it can see the whole site or only its own roles.
//!
//! # Not a security boundary
//!
//! The payload is decoded but **never verified** — no signature check,
//! no issuer check, no clock-skew allowance. A forged token would be
//! described here exactly as it describes itself, and is caught only by
//! the site's manta-server on the next refresh. Treat every field as a
//! diagnostic, never as an authorisation input. This mirrors the
//! standing caveat on `manta_shared::common::jwt_ops`.
//!
//! # Why not `manta_shared::common::jwt_ops`
//!
//! That module does exactly this decode, but depending on
//! `manta-shared` would drag `manta-backend-dispatcher` (+ csm/ochami
//! types, config, utoipa) into an otherwise standalone service. The
//! crate has refused that trade twice already — for `config.rs`'s
//! `ProjectDirs` lookup and `refresh.rs`'s `GroupNode` mirror — and this
//! is the same bargain: a few dozen lines against a large dependency
//! subtree.

use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Keycloak realm role granting site-wide visibility. csm-rs branches on
/// it in `get_group_name_available`: with the role the backend returns
/// every HSM group at the site, without it the caller's available groups
/// *are* its other realm roles. It therefore decides how much of a site
/// this cache can index. Same value as
/// `manta_shared::common::jwt_ops::PA_ADMIN`, duplicated for the reason
/// in the module docs.
const PA_ADMIN: &str = "pa_admin";

/// Prefix Keycloak gives the `preferred_username` of a service account:
/// the username is always `service-account-<client id>`.
const SERVICE_ACCOUNT_PREFIX: &str = "service-account-";

/// How far ahead of expiry to start warning. A service-account token is
/// minted for a year, so the failure it precedes is both far away and
/// easy to forget; a month is enough notice to rotate without being so
/// early that the warning becomes background noise.
pub const EXPIRY_WARNING_DAYS: i64 = 30;

/// Who a credential belongs to, as its own claims describe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
  /// A Keycloak service account — what production should use.
  ///
  /// Identified by convention rather than guesswork: Keycloak sets a
  /// service account's `preferred_username` to `service-account-<azp>`,
  /// so this is an equality test against the client id in the same
  /// token. A human's token carries their login name and cannot satisfy
  /// it.
  ServiceAccount,
  /// A personal login token. Works, but ties a shared service to one
  /// person's roles, expiry, and continued employment.
  User,
  /// Not a JWT, or missing the claims needed to tell the two apart.
  /// Refreshes may still succeed — the backend is the authority on
  /// whether a bearer is valid.
  Unknown,
}

/// Diagnostic view of one site's refresh credential. Built by
/// [`inspect`]; never contains the token itself.
///
/// Not `Serialize`: `GET /dump` renders its own view via
/// `routes::CredentialDump`, which adds the time-relative fields that
/// only make sense at render time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialInfo {
  /// Client id (`azp`) for a service account, login name
  /// (`preferred_username`) for a person, `None` when neither claim
  /// decodes.
  pub principal: Option<String>,
  /// Service account, person, or undecodable.
  pub kind: CredentialKind,
  /// `exp`, when present. Absent for a non-JWT or a token minted
  /// without one.
  pub expires_at: Option<DateTime<Utc>>,
  /// Whether the token carries [`PA_ADMIN`], i.e. whether this site's
  /// snapshot covers every HSM group or only the account's own roles.
  pub is_admin: bool,
}

impl CredentialInfo {
  /// Whole days until expiry — negative once expired, `None` without an
  /// `exp` claim. Computed against a caller-supplied `now` so the value
  /// is fresh at render time rather than frozen at refresh time.
  ///
  /// **Floor** division, not truncation: `TimeDelta::num_days` divides
  /// toward zero, which reports a credential that lapsed six hours ago
  /// as `0` — indistinguishable from one lapsing six hours from now, and
  /// enough to make the obvious `expires_in_days < 0` monitoring alert
  /// miss the entire first day of an outage. Flooring makes a credential
  /// expired by a second or more read `<= -1`, so that alert is right
  /// from the first minute. (`num_seconds` itself truncates, so the
  /// sub-second sliver straight after `exp` still reads `0`; use
  /// [`CredentialInfo::is_expired`] for an exact answer, which is what
  /// the warnings do.)
  pub fn expires_in_days(&self, now: DateTime<Utc>) -> Option<i64> {
    self
      .expires_at
      .map(|exp| (exp - now).num_seconds().div_euclid(86_400))
  }

  /// Has this credential's `exp` passed? `false` when it carries none.
  ///
  /// Asks the timestamps directly rather than going through
  /// [`CredentialInfo::expires_in_days`], so the answer is exact at any
  /// point inside the day of expiry.
  pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
    self.expires_at.is_some_and(|exp| exp <= now)
  }

  /// Operator-facing problems with this credential, worst first. Empty
  /// for a healthy service-account token.
  ///
  /// Callers log these and `GET /dump` renders them; nothing acts on
  /// them. Pair with [`CredentialInfo::is_expired`] to pick a log level
  /// — an expired credential is an outage, not a heads-up.
  pub fn warnings(&self, now: DateTime<Utc>) -> Vec<String> {
    let mut out = Vec::new();

    if self.is_expired(now) {
      // Dated rather than counted: "expired 0 day(s) ago" is what a
      // day-count says for the first 24 hours, which reads as though
      // nothing has happened yet.
      out.push(format!(
        "credential expired on {} — refreshes will fail until it is \
         replaced",
        self
          .expires_at
          .expect("is_expired is only true with an exp")
          .format("%Y-%m-%d %H:%M UTC")
      ));
    } else if let Some(days) = self.expires_in_days(now)
      && days <= EXPIRY_WARNING_DAYS
    {
      out.push(format!(
        "credential expires in {days} day(s) — replace the token file \
         before it lapses (no restart needed; it is re-read every refresh)"
      ));
    }

    match self.kind {
      CredentialKind::User => out.push(format!(
        "refreshing as personal account '{}' — production should use a \
         per-site Keycloak service account, whose roles and lifetime do \
         not follow one person",
        self.principal.as_deref().unwrap_or("<unknown>")
      )),
      // Split by what was actually recoverable, so the message never
      // contradicts the expiry the summary line prints beside it: a JWT
      // whose identity claims are missing still yields a usable `exp`,
      // and claiming otherwise would send an operator looking for a
      // decode problem that does not exist.
      CredentialKind::Unknown if self.expires_at.is_some() => out.push(
        "credential carries no recognisable principal claims — a service \
         account cannot be told apart from a personal token here, though \
         its expiry below is readable"
          .to_string(),
      ),
      CredentialKind::Unknown => out.push(
        "credential is not a decodable JWT — neither its principal nor \
         its expiry can be checked, so a lapse will surface only as a \
         failed refresh"
          .to_string(),
      ),
      CredentialKind::ServiceAccount => {}
    }

    out
  }

  /// One-line rendering for the startup summary, e.g.
  /// `service account 'manta-cache-test', scoped to its own roles,
  /// expires 2027-08-05 (364 days)`.
  pub fn summary(&self, now: DateTime<Utc>) -> String {
    // Built as a prefix + the shared expiry suffix rather than returning
    // early for `Unknown`: a token can decode, carry an `exp`, and still
    // lack the identity claims (a Keycloak client with no username
    // mapper), and the expiry date is the single most useful thing this
    // line reports. Dropping it for those tokens would hide the one fact
    // worth printing.
    let who = match (self.kind, self.principal.as_deref()) {
      (CredentialKind::ServiceAccount, Some(p)) => {
        format!("service account '{p}', {}", self.scope())
      }
      (CredentialKind::ServiceAccount, None) => {
        format!("service account, {}", self.scope())
      }
      (CredentialKind::User, Some(p)) => {
        format!("PERSONAL account '{p}', {}", self.scope())
      }
      (CredentialKind::User, None) => {
        format!("PERSONAL account, {}", self.scope())
      }
      // Scope is appended only when the token actually claims admin:
      // saying "scoped to its own roles" about a credential whose roles
      // never decoded would assert something unknown, but a decoded
      // `pa_admin` is worth reporting — and omitting it entirely would
      // leave the summary disagreeing with `is_admin` on `/dump`.
      (CredentialKind::Unknown, _) if self.is_admin => {
        format!("unidentified credential, {}", self.scope())
      }
      (CredentialKind::Unknown, _) => "unidentified credential".to_string(),
    };
    let expiry = match (self.expires_at, self.expires_in_days(now)) {
      (Some(at), Some(days)) if self.is_expired(now) => {
        format!(", EXPIRED {} ({} day(s) ago)", at.format("%Y-%m-%d"), -days)
      }
      (Some(at), Some(days)) => {
        format!(", expires {} ({days} days)", at.format("%Y-%m-%d"))
      }
      _ => ", no expiry claim".to_string(),
    };
    format!("{who}{expiry}")
  }

  /// How much of a site this credential can see, in words.
  fn scope(&self) -> &'static str {
    if self.is_admin {
      "site-wide (pa_admin)"
    } else {
      "scoped to its own roles"
    }
  }
}

/// Describe `token` from its own claims.
///
/// Total by construction: anything that does not decode as a JWT with
/// the expected claims yields [`CredentialKind::Unknown`] rather than an
/// error. A credential the cache cannot describe is still a credential
/// the cache should try to refresh with — the backend decides.
pub fn inspect(token: &str) -> CredentialInfo {
  let Some(claims) = decode_claims(token) else {
    return CredentialInfo {
      principal: None,
      kind: CredentialKind::Unknown,
      expires_at: None,
      is_admin: false,
    };
  };

  let azp = claims.get("azp").and_then(|v| v.as_str());
  let username = claims.get("preferred_username").and_then(|v| v.as_str());

  // Keycloak's own naming convention is the test: a service account's
  // username is derived from its client id, which no human login can
  // reproduce.
  let is_service_account = match (azp, username) {
    (Some(client), Some(user)) => {
      user == format!("{SERVICE_ACCOUNT_PREFIX}{client}")
    }
    _ => false,
  };

  let (kind, principal) = if is_service_account {
    (CredentialKind::ServiceAccount, azp)
  } else if username.is_some() {
    (CredentialKind::User, username)
  } else {
    (CredentialKind::Unknown, None)
  };

  let is_admin = claims
    .pointer("/realm_access/roles")
    .and_then(|v| v.as_array())
    .is_some_and(|roles| roles.iter().any(|r| r.as_str() == Some(PA_ADMIN)));

  CredentialInfo {
    principal: principal.map(str::to_owned),
    kind,
    expires_at: claims
      .get("exp")
      .and_then(serde_json::Value::as_i64)
      .and_then(|secs| DateTime::from_timestamp(secs, 0)),
    is_admin,
  }
}

/// Decode a JWT's payload segment without verifying anything. `None`
/// for any input that is not `header.payload.signature` with a
/// base64-encoded JSON object in the middle.
///
/// Accepts both base64url (what Keycloak emits) and standard base64,
/// padded or not — the same leniency `manta_shared`'s decoder applies,
/// since tokens are copied by hand into secret stores and occasionally
/// arrive re-encoded.
fn decode_claims(token: &str) -> Option<serde_json::Value> {
  let mut segments = token.split('.');
  let (_header, payload, signature) =
    (segments.next()?, segments.next()?, segments.next()?);
  if signature.is_empty() || segments.next().is_some() {
    return None;
  }

  let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
    .decode(payload)
    .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(payload))
    .or_else(|_| base64::engine::general_purpose::STANDARD.decode(payload))
    .ok()?;

  // Only an object carries claims; a bare array or string decoding
  // cleanly would otherwise be reported as a claimless JWT.
  match serde_json::from_slice::<serde_json::Value>(&bytes).ok()? {
    value @ serde_json::Value::Object(_) => Some(value),
    _ => None,
  }
}

/// Token builders shared by this module's tests and `routes`'/
/// `refresh`'s, which need a realistic credential on disk.
#[cfg(test)]
pub mod test_support {
  use super::*;

  /// Build an unsigned JWT carrying `claims`, in the base64url form
  /// Keycloak emits.
  pub fn jwt(claims: serde_json::Value) -> String {
    let encode =
      |raw: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);
    format!(
      "{}.{}.sig",
      encode(br#"{"alg":"RS256","typ":"JWT"}"#),
      encode(claims.to_string().as_bytes())
    )
  }

  /// The shape of a real prealps service-account token (see ROADMAP
  /// Stage 5), minus the claims this module ignores.
  pub fn service_account_token(exp: DateTime<Utc>) -> String {
    jwt(serde_json::json!({
      "typ": "Bearer",
      "azp": "manta-cache-test",
      "preferred_username": "service-account-manta-cache-test",
      "realm_access": { "roles": [
        "default-roles-shasta", "meda", "offline_access",
        "uma_authorization", "gallina"
      ]},
      "exp": exp.timestamp(),
    }))
  }

  /// A service-account token valid for `days` from now.
  pub fn service_account_token_valid_for(days: i64) -> String {
    service_account_token(Utc::now() + chrono::Duration::days(days))
  }
}

#[cfg(test)]
mod tests {
  use super::test_support::*;
  use super::*;

  fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_800_000_000, 0).unwrap()
  }

  #[test]
  fn identifies_a_service_account_from_the_username_convention() {
    let info =
      inspect(&service_account_token(now() + chrono::Duration::days(300)));
    assert_eq!(info.kind, CredentialKind::ServiceAccount);
    assert_eq!(info.principal.as_deref(), Some("manta-cache-test"));
    // The real account holds `meda` + `gallina`, not pa_admin — so the
    // cache indexes only those groups.
    assert!(!info.is_admin);
    assert!(info.warnings(now()).is_empty());
  }

  #[test]
  fn identifies_a_personal_token_and_warns_about_it() {
    let info = inspect(&jwt(serde_json::json!({
      "azp": "manta",
      "preferred_username": "alice",
      "name": "Alice Smith",
    })));
    assert_eq!(info.kind, CredentialKind::User);
    assert_eq!(info.principal.as_deref(), Some("alice"));
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(
      warnings[0].contains("personal account 'alice'"),
      "{warnings:?}"
    );
  }

  #[test]
  fn a_username_matching_another_clients_service_account_is_not_one() {
    // `preferred_username` follows the convention but names a
    // *different* client than `azp`: the equality must be against this
    // token's own client id, not the prefix alone.
    let info = inspect(&jwt(serde_json::json!({
      "azp": "manta",
      "preferred_username": "service-account-something-else",
    })));
    assert_eq!(info.kind, CredentialKind::User);
  }

  #[test]
  fn detects_the_admin_role() {
    let info = inspect(&jwt(serde_json::json!({
      "azp": "c",
      "preferred_username": "service-account-c",
      "realm_access": { "roles": ["pa_admin", "meda"] },
    })));
    assert!(info.is_admin);
    assert!(info.summary(now()).contains("site-wide"));
  }

  #[test]
  fn warns_within_the_expiry_window_but_not_before() {
    let comfortable =
      inspect(&service_account_token(now() + chrono::Duration::days(90)));
    assert!(comfortable.warnings(now()).is_empty());

    let soon = inspect(&service_account_token(
      now() + chrono::Duration::days(EXPIRY_WARNING_DAYS - 1),
    ));
    let warnings = soon.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("expires in 29 day(s)"), "{warnings:?}");
  }

  #[test]
  fn reports_an_expired_credential_as_expired() {
    let info =
      inspect(&service_account_token(now() - chrono::Duration::days(3)));
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("expired on"), "{warnings:?}");
    assert!(info.is_expired(now()));
    assert_eq!(info.expires_in_days(now()), Some(-3));
  }

  #[test]
  fn a_credential_that_lapsed_hours_ago_is_not_reported_as_still_valid() {
    // `TimeDelta::num_days` truncates toward zero, so the raw day count
    // for a credential six hours past `exp` is 0 — which read as "expires
    // in 0 day(s) — replace it before it lapses" for a credential that
    // already had, during exactly the 24 hours someone is trying to work
    // out why refreshes started failing.
    let info =
      inspect(&service_account_token(now() - chrono::Duration::hours(6)));

    assert!(info.is_expired(now()));
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("expired on"), "{warnings:?}");
    assert!(!warnings[0].contains("before it lapses"), "{warnings:?}");

    // And the field monitoring is pointed at must be negative, so the
    // obvious `expires_in_days < 0` alert does not miss the first day.
    assert_eq!(info.expires_in_days(now()), Some(-1));
    assert!(info.summary(now()).contains("EXPIRED"));
  }

  #[test]
  fn a_credential_lapsing_later_today_still_reads_as_valid() {
    // The mirror case: six hours *before* expiry must not be reported as
    // expired, even though it shares the same truncated day count.
    let info =
      inspect(&service_account_token(now() + chrono::Duration::hours(6)));
    assert!(!info.is_expired(now()));
    assert_eq!(info.expires_in_days(now()), Some(0));
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("expires in 0 day(s)"), "{warnings:?}");
  }

  #[test]
  fn a_jwt_without_identity_claims_still_reports_its_expiry() {
    // A Keycloak client with no username mapper decodes fine and carries
    // an `exp`; returning early on `Unknown` would drop the one fact the
    // summary line exists to convey.
    let info = inspect(&jwt(serde_json::json!({
      "sub": "1234",
      "exp": (now() + chrono::Duration::days(400)).timestamp(),
    })));
    assert_eq!(info.kind, CredentialKind::Unknown);
    let summary = info.summary(now());
    assert!(summary.contains("expires"), "{summary}");
    assert!(summary.contains("400 days"), "{summary}");

    // And the warning must not contradict the line above it by claiming
    // the expiry is unreadable.
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 1);
    assert!(
      warnings[0].contains("no recognisable principal"),
      "{warnings:?}"
    );
    assert!(!warnings[0].contains("not a decodable JWT"), "{warnings:?}");
  }

  #[test]
  fn a_credential_without_an_exp_claim_says_so() {
    let info = inspect(&jwt(serde_json::json!({
      "azp": "c",
      "preferred_username": "service-account-c",
    })));
    assert_eq!(info.expires_in_days(now()), None);
    assert!(!info.is_expired(now()));
    assert!(info.summary(now()).contains("no expiry claim"));
    // No expiry is not itself a problem — the backend remains the
    // authority on whether the bearer works.
    assert!(info.warnings(now()).is_empty());
  }

  #[test]
  fn a_personal_token_near_expiry_reports_both_problems() {
    let info = inspect(&jwt(serde_json::json!({
      "azp": "manta",
      "preferred_username": "alice",
      "exp": (now() + chrono::Duration::days(2)).timestamp(),
    })));
    // Naming both: a bare length check would pass if the same problem
    // were emitted twice.
    let warnings = info.warnings(now());
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings[0].contains("expires in 2 day(s)"), "{warnings:?}");
    assert!(
      warnings[1].contains("personal account 'alice'"),
      "{warnings:?}"
    );
  }

  #[test]
  fn non_jwt_credentials_are_unknown_not_errors() {
    for opaque in [
      "",
      "opaque-api-key",
      "two.segments",
      "a.b.c.d",
      "header..sig",
      // A filesystem path: the exact mistake of pointing `token_file`
      // at a path *containing* the token rather than at the token.
      "/Users/x/Library/Caches/local.cscs.manta/prealps_auth",
    ] {
      let info = inspect(opaque);
      assert_eq!(
        info.kind,
        CredentialKind::Unknown,
        "expected {opaque:?} to be undecodable"
      );
      assert_eq!(info.warnings(now()).len(), 1);
    }
  }

  #[test]
  fn a_jwt_without_identity_claims_is_unknown() {
    let info = inspect(&jwt(serde_json::json!({ "sub": "1234" })));
    assert_eq!(info.kind, CredentialKind::Unknown);
    assert!(info.principal.is_none());
  }

  #[test]
  fn summary_never_contains_the_token() {
    let token = service_account_token(now() + chrono::Duration::days(365));
    let info = inspect(&token);
    let rendered = format!("{} {:?}", info.summary(now()), info);
    for segment in token.split('.') {
      assert!(!rendered.contains(segment), "{rendered}");
    }
  }
}
