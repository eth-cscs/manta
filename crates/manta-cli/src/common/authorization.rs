//! Authorization helpers: validate user access to HSM groups and their members.

use manta_backend_dispatcher::{error::Error, interfaces::hsm::group::GroupTrait};

use manta_shared::manta_backend_dispatcher::StaticBackendDispatcher;

/// Return the accessible HSM groups for the token; errors if the requested group is not accessible.
pub async fn get_groups_names_available(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  group_cli_arg_opt: Option<&str>,
  group_env_or_config_file_opt: Option<&str>,
) -> Result<Vec<String>, Error> {
  let hsm_name_available_vec =
    backend.get_group_name_available(auth_token).await?;

  let target_hsm_group_opt = if group_cli_arg_opt.is_some() {
    group_cli_arg_opt
  } else {
    group_env_or_config_file_opt
  };

  if let Some(target_hsm_group) = target_hsm_group_opt {
    if !hsm_name_available_vec
      .iter()
      .any(|name| name == target_hsm_group)
    {
      let mut hsm_name_available_vec = hsm_name_available_vec;
      hsm_name_available_vec.sort();
      return Err(Error::BadRequest(format!(
        "Can't access HSM group '{}'.\nPlease choose one \
         from the list below:\n{}",
        target_hsm_group,
        hsm_name_available_vec.join(", ")
      )));
    }

    Ok(vec![target_hsm_group.to_string()])
  } else {
    Ok(hsm_name_available_vec)
  }
}

/// Validate that every requested xname belongs to a group the token has access to.
pub async fn validate_target_hsm_members(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hsm_group_members_opt: &[String],
) -> Result<Vec<String>, Error> {
  let hsm_groups_user_has_access =
    backend.get_group_name_available(shasta_token).await?;

  let all_xnames_user_has_access = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      &hsm_groups_user_has_access,
    )
    .await?;

  if hsm_group_members_opt
    .iter()
    .all(|hsm_member| all_xnames_user_has_access.contains(hsm_member))
  {
    Ok(hsm_group_members_opt.to_vec())
  } else {
    Err(Error::BadRequest(format!(
      "Can't access all or any of the HSM members \
       '{}'.\nPlease choose members form the list \
       of HSM groups below:\n{}",
      hsm_group_members_opt.join(", "),
      hsm_groups_user_has_access.join(", ")
    )))
  }
}
