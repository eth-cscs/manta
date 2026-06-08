//! Authorization helpers: validate user access to HSM groups and their members.

use manta_backend_dispatcher::error::Error;

use crate::server::common::{app_context::InfraContext, jwt_ops};

/// Keycloak role name that grants full admin access (bypasses HSM-group
/// scoping checks).
pub static PA_ADMIN: &str = "pa_admin";

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
  if jwt_ops::is_user_admin(token) {
    return Ok(());
  }

  let group_available_vec = infra.get_group_name_available(token).await?;

  validate_group_vec_access(&[group_name.to_string()], &group_available_vec)
}

/// Validate if user has access to a list of target groups
pub async fn validate_user_group_vec_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_vec: &[String],
) -> Result<(), Error> {
  if jwt_ops::is_user_admin(token) {
    return Ok(());
  }

  let group_available_vec = infra.get_group_name_available(token).await?;

  validate_group_vec_access(group_vec, &group_available_vec)
}

/// Checks if a list of target groups belongs to the list of groups the user has access to
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

/// Validate that every xname in a comma-separated `ansible_limit`-style
/// string belongs to a group the token has access to.
pub async fn validate_ansible_limit_membership_access(
  infra: &InfraContext<'_>,
  token: &str,
  ansible_limit: &str,
) -> Result<(), Error> {
  if jwt_ops::is_user_admin(token) {
    return Ok(());
  }

  let xnames: Vec<String> = ansible_limit
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();
  validate_user_group_members_access(infra, token, &xnames).await
}

/// Validate that every requested xname belongs to a group the token has access to.
pub async fn validate_user_group_members_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_members_target_vec: &[String],
) -> Result<(), Error> {
  if jwt_ops::is_user_admin(token) {
    return Ok(());
  }

  let hsm_groups_user_has_access =
    infra.get_group_name_available(token).await?;

  validate_group_members_access(
    infra,
    token,
    group_members_target_vec,
    &hsm_groups_user_has_access,
  )
  .await
}

/// Validate that every requested xname belongs to a group the token has access to.
pub async fn validate_group_members_access(
  infra: &InfraContext<'_>,
  token: &str,
  group_members_target_vec: &[String],
  hsm_groups_user_has_access: &[String],
) -> Result<(), Error> {
  if jwt_ops::is_user_admin(token) {
    return Ok(());
  }

  let all_xnames_user_has_access = infra
    .get_member_vec_from_group_name_vec(token, hsm_groups_user_has_access)
    .await?;

  let invalid_xnames: Vec<String> = group_members_target_vec
    .iter()
    .filter(|group| !all_xnames_user_has_access.contains(group))
    .cloned()
    .collect();

  if invalid_xnames.is_empty() {
    Ok(())
  } else {
    Err(Error::BadRequest(format!(
      "Invalid group members:\n'{:?}'.\nPlease choose members form the list of groups below:\n{}",
      invalid_xnames,
      hsm_groups_user_has_access.join(", ")
    )))
  }
}
