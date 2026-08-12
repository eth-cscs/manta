//! `SessionContext` — per-invocation facts derived from the bearer
//! token, plus the one server round-trip that lists the groups the
//! token can reach.
//!
//! Built once at the top of `crate::dispatch::process::process_cli`
//! for every command that authenticates, then attached to
//! [`crate::common::app_context::AppContext::session`] so handlers
//! and the read-only gate can read JWT-derived state without
//! re-decoding the token or re-fetching the groups list.

use anyhow::Context;
use manta_shared::common::jwt_ops;

use crate::http_client::{MantaClient, OpenApiResultExt};

/// Facts derived from the bearer token (and one `GET
/// /groups/available` round-trip) that aren't in `cli.toml`. Built
/// once per CLI invocation right after the auth cascade resolves a
/// valid token.
#[derive(Debug, Clone)]
pub struct SessionContext {
  /// `preferred_username` claim — usually the operator's login name.
  pub username: String,
  /// `name` claim — the operator's display name.
  pub name: String,
  /// `true` when `realm_access.roles` carries
  /// [`manta_shared::common::jwt_ops::PA_ADMIN`].
  pub is_admin: bool,
  /// Groups visible to the token. For non-admin tokens the server
  /// filters by realm role; for admin tokens it returns the full
  /// group universe.
  pub accessible_groups: Vec<String>,
}

impl SessionContext {
  /// Pure constructor — no I/O. Split out from [`SessionContext::build`]
  /// so tests can exercise the JWT-decode path without HTTP stubbing.
  fn from_parts(
    token: &str,
    accessible_groups: Vec<String>,
  ) -> anyhow::Result<Self> {
    let username = jwt_ops::get_preferred_username(token)
      .context("decode preferred_username from token")?;
    let name = jwt_ops::get_name(token).context("decode name from token")?;
    let roles = jwt_ops::get_roles(token)
      .context("decode realm_access.roles from token")?;
    let is_admin = roles.iter().any(|r| r == jwt_ops::PA_ADMIN);
    Ok(Self {
      username,
      name,
      is_admin,
      accessible_groups,
    })
  }

  /// Build a `SessionContext` by:
  ///
  /// 1. Calling `GET /v2/groups/available` to populate
  ///    `accessible_groups` (server filters by realm role for
  ///    non-admin callers, returns the universe for admins).
  /// 2. Locally decoding the JWT claims via
  ///    [`manta_shared::common::jwt_ops`].
  ///
  /// Returns `Err` when the HTTP call or the JWT decode fails.
  ///
  /// # Errors
  ///
  /// - HTTP failure on `get_available_groups` (network, 401/403,
  ///   server error).
  /// - JWT decode failure (malformed token, missing claim).
  pub async fn build(
    client: &MantaClient,
    token: &str,
  ) -> anyhow::Result<Self> {
    let accessible_groups = client
      .openapi
      .get_available_groups(client.site_name())
      .await
      .into_anyhow()
      .await
      .context("fetch accessible groups")?;
    Self::from_parts(token, accessible_groups)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use base64::Engine as _;

  /// Build a base64url-encoded JWT with the given claims. The header
  /// is `{"alg":"none"}` and the signature segment is a stub — the
  /// `jwt_ops` decoder doesn't verify it. Mirrors the fixture style
  /// in `manta_shared::common::jwt_ops::tests`.
  fn jwt(name: &str, username: &str, roles: &[&str]) -> String {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .encode(br#"{"alg":"none"}"#);
    let roles_json = serde_json::to_string(roles).unwrap();
    let payload_json = format!(
      r#"{{"name":"{name}","preferred_username":"{username}","realm_access":{{"roles":{roles_json}}}}}"#
    );
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .encode(payload_json.as_bytes());
    format!("{header}.{payload}.sig")
  }

  #[test]
  fn from_parts_populates_all_fields_from_admin_token() {
    let token = jwt("Alice Smith", "alice", &["pa_admin", "compute"]);
    let session = SessionContext::from_parts(
      &token,
      vec!["alps".to_string(), "compute".to_string()],
    )
    .expect("build session");
    assert_eq!(session.username, "alice");
    assert_eq!(session.name, "Alice Smith");
    assert!(session.is_admin);
    assert_eq!(
      session.accessible_groups,
      vec!["alps".to_string(), "compute".to_string()]
    );
  }

  #[test]
  fn from_parts_marks_non_admin_for_plain_user() {
    let token = jwt("Carol", "carol", &["compute"]);
    let session = SessionContext::from_parts(&token, vec![]).unwrap();
    assert!(!session.is_admin);
  }

  #[test]
  fn from_parts_propagates_jwt_decode_failure() {
    let result = SessionContext::from_parts("not.a.jwt", vec![]);
    assert!(result.is_err(), "expected Err on malformed JWT");
  }
}
