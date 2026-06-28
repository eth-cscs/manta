//! Authorization helpers: validate user access to HSM groups and
//! their members.
//!
//! Every service-layer function that takes a node, group, or session
//! label from the caller runs one of these checks before touching the
//! backend. The standard pattern is:
//!
//! 1. Resolve the caller's request to a `Vec<String>` of xnames or
//!    group labels (often via [`crate::service::node_ops`]).
//! 2. Call [`validate_user_group_members_access`] (xnames) or
//!    [`validate_user_group_vec_access`] (group labels).
//! 3. Proceed to the actual backend mutation.
//!
//! Admin tokens carrying the [`PA_ADMIN`] role short-circuit every
//! check to `Ok(())` without touching the backend, mirroring the
//! "admin sees everything" expectation. Listing endpoints still
//! validate so the response can't disclose more than the caller
//! could have asked for directly.
//!
//! The short-circuit is centralised in the private [`admin_bypass`]
//! helper so that a future change — e.g. adding audit logging for
//! admin bypasses, or gating on JWKS verification before skipping
//! group-scope checks — only needs to touch one place.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::server::common::{app_context::InfraContext, jwt_ops};

/// Keycloak role name that grants full admin access (bypasses HSM-group
/// scoping checks).
pub static PA_ADMIN: &str = "pa_admin";

/// Returns `true` when the caller is admin (carries `PA_ADMIN` in
/// `realm_access.roles`). Admin tokens short-circuit every group-scope
/// check in this module — see the module-level security note and the
/// `jwt_ops.rs` module doc for the no-local-signature-verification posture.
fn admin_bypass(token: &str) -> bool {
  jwt_ops::is_user_admin(token)
}

/// Validate that `group_name` is in the set this token can access.
///
/// Used by handlers that perform privileged HSM-group operations and
/// need a server-side authorization check before delegating to the
/// service layer. Returns `Error::BadRequest` with a usable error
/// message when the group is not accessible.
pub async fn validate_user_group_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_name: &str,
) -> Result<(), Error> {
  if admin_bypass(token) {
    return Ok(());
  }

  let group_available_vec =
    infra.backend.get_group_name_available(token).await?;

  validate_group_vec_access(&[group_name.to_string()], &group_available_vec)
}

/// Validate that every label in `group_vec` is in the set the token
/// can access.
///
/// Admin tokens (carrying the [`PA_ADMIN`] role) short-circuit to
/// `Ok` without touching the backend. Otherwise the available-group
/// list is fetched once and matched against `group_vec`. Use the
/// single-group variant [`validate_user_group_access`] when you only
/// need to check one label.
pub async fn validate_user_group_vec_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_vec: &[String],
) -> Result<(), Error> {
  if admin_bypass(token) {
    return Ok(());
  }

  let group_available_vec =
    infra.backend.get_group_name_available(token).await?;

  validate_group_vec_access(group_vec, &group_available_vec)
}

/// Pure check that every label in `group_target_vec` appears in
/// `group_available_vec`.
///
/// The async wrappers above resolve `group_available_vec` from the
/// backend; this entry point exists for callers that already have
/// the available list in hand (or for unit tests). On failure the
/// `BadRequest` message lists the offending labels followed by the
/// allowed set, so the user gets an actionable hint without a second
/// round-trip.
pub fn validate_group_vec_access(
  group_target_vec: &[String],
  group_available_vec: &[String],
) -> Result<(), Error> {
  let mut invalid_group_vec: Vec<String> = group_target_vec
    .iter()
    .filter(|group| !group_available_vec.contains(group))
    .cloned()
    .collect();

  if invalid_group_vec.is_empty() {
    Ok(())
  } else {
    invalid_group_vec.sort();

    Err(Error::BadRequest(format!(
      "Invalid groups '{:?}'.\nPlease choose one from the list below:\n{}",
      invalid_group_vec,
      group_available_vec.join(", ")
    )))
  }
}

/// Validate every xname in a comma-separated `ansible_limit`-style
/// string against the caller's accessible groups.
///
/// Splits on `,`, trims, and forwards to
/// [`validate_user_group_members_access`]. Admin tokens skip the
/// check entirely. Use this at handler boundaries where the request
/// shape is the raw ansible-limit string (e.g. CFS session creation).
pub async fn validate_ansible_limit_membership_access(
  infra: &InfraContext<'_>,
  token: &str,
  ansible_limit: &str,
) -> Result<(), Error> {
  if admin_bypass(token) {
    return Ok(());
  }

  let xnames: Vec<String> = ansible_limit
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();
  validate_user_group_members_access(infra, token, &xnames).await
}

/// Validate that every xname in `group_members_target_vec` is a
/// member of at least one group the token can access.
///
/// Admin tokens skip the check. Otherwise the caller's accessible
/// group list is fetched, expanded to member xnames, and matched
/// against the request. This is the standard membership gate used by
/// the per-node and per-host service helpers.
pub async fn validate_user_group_members_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_members_target_vec: &[String],
) -> Result<(), Error> {
  if admin_bypass(token) {
    return Ok(());
  }

  let hsm_groups_user_has_access =
    infra.backend.get_group_name_available(token).await?;

  validate_group_members_access(
    infra,
    token,
    group_members_target_vec,
    &hsm_groups_user_has_access,
  )
  .await
}

/// Like [`validate_user_group_members_access`] but with the
/// caller-accessible group list supplied explicitly.
///
/// Lets a caller that has already fetched `hsm_groups_user_has_access`
/// reuse it across several membership checks without an extra
/// round-trip. Admin tokens still short-circuit.
pub async fn validate_group_members_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_members_target_vec: &[String],
  hsm_groups_user_has_access: &[String],
) -> Result<(), Error> {
  if admin_bypass(token) {
    return Ok(());
  }

  let all_xnames_user_has_access = infra
    .backend
    .get_member_vec_from_group_name_vec(token, hsm_groups_user_has_access)
    .await?;

  // Hash the accessible-xname set once. It can be cluster-scale (every
  // xname in every group the caller can see), so the previous
  // `.contains()` per target was O(target_count · accessible_count).
  let accessible_set: std::collections::HashSet<&str> =
    all_xnames_user_has_access
      .iter()
      .map(String::as_str)
      .collect();
  let invalid_xnames: Vec<String> = group_members_target_vec
    .iter()
    .filter(|group| !accessible_set.contains(group.as_str()))
    .cloned()
    .collect();

  if invalid_xnames.is_empty() {
    Ok(())
  } else {
    Err(Error::BadRequest(format!(
      "Invalid group members:\n'{:?}'.\nPlease choose members from the list of groups below:\n{}",
      invalid_xnames,
      hsm_groups_user_has_access.join(", ")
    )))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
  }

  #[test]
  fn allows_when_every_target_is_in_available_set() {
    let result = validate_group_vec_access(
      &s(&["compute", "login"]),
      &s(&["compute", "login", "storage"]),
    );
    assert!(result.is_ok(), "got {result:?}");
  }

  #[test]
  fn allows_empty_target_set() {
    let result = validate_group_vec_access(&[], &s(&["compute"]));
    assert!(result.is_ok(), "got {result:?}");
  }

  #[test]
  fn rejects_when_any_target_is_missing_from_available_set() {
    let err =
      validate_group_vec_access(&s(&["compute", "secret"]), &s(&["compute"]))
        .unwrap_err();
    let Error::BadRequest(msg) = err else {
      panic!("expected BadRequest, got {err:?}");
    };
    assert!(
      msg.contains("\"secret\""),
      "error message should name the offending group: {msg}"
    );
    assert!(
      !msg.contains("\"compute\""),
      "error message should not name the allowed group: {msg}"
    );
  }

  #[test]
  fn rejects_when_available_set_is_empty() {
    let err = validate_group_vec_access(&s(&["compute"]), &[]).unwrap_err();
    assert!(matches!(err, Error::BadRequest(_)));
  }

  // Sorting the offending list keeps the error message deterministic
  // across runs — important for CLI users grepping their failure log.
  #[test]
  fn error_message_sorts_offending_groups_alphabetically() {
    let err =
      validate_group_vec_access(&s(&["zeta", "alpha", "mu"]), &s(&["other"]))
        .unwrap_err();
    let Error::BadRequest(msg) = err else {
      panic!("expected BadRequest, got {err:?}");
    };
    let alpha = msg.find("alpha").expect("alpha listed");
    let mu = msg.find("mu").expect("mu listed");
    let zeta = msg.find("zeta").expect("zeta listed");
    assert!(alpha < mu && mu < zeta, "got: {msg}");
  }
}
