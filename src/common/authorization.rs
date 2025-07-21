use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Returns a curated list of 'groups' the user has access to.
/// This function validates the list of groups and returns an error if user tries to access a
/// group he does not have access to
/// If the user did not request any HSM group, then it will return all groups available
pub async fn get_groups_available(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  group_cli_arg_opt: Option<&String>,
  group_env_or_config_file_opt: Option<&String>,
) -> Result<Vec<String>, manta_backend_dispatcher::error::Error> {
  // Get list of groups the user has access to
  let hsm_name_available_vec =
    backend.get_group_name_available(auth_token).await?;

  // Get the group name the user is trying to work with, this value can be in 2 different places:
  //  - command argument
  //  - configuration (environment variable or config file)
  let target_hsm_group_opt = if group_cli_arg_opt.is_some() {
    group_cli_arg_opt
  } else {
    group_env_or_config_file_opt
  };

  // Validate the user has access to the HSM group is requested
  if let Some(target_hsm_group) = target_hsm_group_opt {
    if !hsm_name_available_vec.contains(target_hsm_group) {
      let mut hsm_name_available_vec = hsm_name_available_vec;
      hsm_name_available_vec.sort();
      println!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_group,
                hsm_name_available_vec.join(", ")
            );

      std::process::exit(1);
    }

    Ok(vec![target_hsm_group.to_string()])
  } else {
    Ok(hsm_name_available_vec)
  }
}

/// Validate user has access to a list of HSM group members provided.
/// HSM members user is asking for are taken from cli command
/// Exit if user does not have access to any of the members provided. By not having access to a HSM
/// members means, the node belongs to an HSM group which the user does not have access
pub async fn validate_target_hsm_members(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hsm_group_members_opt: &Vec<String>,
) -> Vec<String> {
  let hsm_groups_user_has_access = backend
    .get_group_name_available(shasta_token)
    .await
    .unwrap();

  let all_xnames_user_has_access = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      hsm_groups_user_has_access.clone(),
    )
    .await
    .unwrap();

  // Check user has access to all xnames he is requesting
  if hsm_group_members_opt
    .iter()
    .all(|hsm_member| all_xnames_user_has_access.contains(hsm_member))
  {
    hsm_group_members_opt.to_vec()
  } else {
    println!("Can't access all or any of the HSM members '{}'.\nPlease choose members form the list of HSM groups below:\n{}\nExit", hsm_group_members_opt.join(", "), hsm_groups_user_has_access.join(", "));
    std::process::exit(1);
  }
}
