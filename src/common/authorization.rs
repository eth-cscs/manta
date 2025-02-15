use backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::backend_dispatcher::StaticBackendDispatcher;

/// Returns a curated list of 'groups' the user has access to.
/// This function validates the list of groups and returns an error if user tries to access a
/// group he does not have access to
/// If the user did not request any HSM group, then it will return all groups available
pub async fn get_groups_available(
    backend: &StaticBackendDispatcher,
    auth_token: &str,
    group_cli_arg_opt: Option<&String>,
    group_env_or_config_file_opt: Option<&String>,
) -> Result<Vec<String>, backend_dispatcher::error::Error> {
    // Get list of groups the user has access to
    let hsm_name_available_vec = backend.get_group_name_available(auth_token).await?;

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

/* /// Returns a list of HSM groups the user is expected to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// If the user did not requested any HSM group, then it will return all HSM
/// groups he has access to
pub async fn get_target_hsm_group_vec_or_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_cli_arg_opt: Option<&String>,
    hsm_group_env_or_config_file_opt: Option<&String>,
) -> Vec<String> {
    // Get list of groups the user has access to
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    // Get the group name the user is trying to work with, this value can be in 2 different places:
    //  - command argument
    //  - configuration (environment variable or config file)
    let target_hsm_group_opt = if hsm_group_cli_arg_opt.is_some() {
        hsm_group_cli_arg_opt
    } else {
        hsm_group_env_or_config_file_opt
    };

    // Validate the user has access to the HSM group is requested
    if let Some(target_hsm_group) = target_hsm_group_opt {
        if !hsm_name_available_vec.contains(target_hsm_group) {
            println!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_group,
                hsm_name_available_vec.join(", ")
            );

            std::process::exit(1);
        }

        vec![target_hsm_group.to_string()]
    } else {
        hsm_name_available_vec
    }
} */

/// Validate user has access to a list of HSM group members provided.
/// HSM members user is asking for are taken from cli command
/// Exit if user does not have access to any of the members provided. By not having access to a HSM
/// members means, the node belongs to an HSM group which the user does not have access
pub async fn validate_target_hsm_members(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    /* shasta_base_url: &str,
    shasta_root_cert: &[u8], */
    hsm_group_members_opt: &Vec<String>,
) -> Vec<String> {
    let hsm_groups_user_has_access = backend
        .get_group_name_available(shasta_token)
        .await
        .unwrap();
    /* let hsm_groups_user_has_access = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await; */

    let all_xnames_user_has_access = backend
        .get_member_vec_from_group_name_vec(
            shasta_token,
            /* shasta_base_url,
            shasta_root_cert, */
            hsm_groups_user_has_access.clone(),
        )
        .await
        .unwrap();
    /* let all_xnames_user_has_access = hsm::group::utils::get_member_vec_from_hsm_name_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_groups_user_has_access.clone(),
    )
    .await; */

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

/* pub fn validate_hsm_groups(
    target_hsm_name_vec: &Vec<String>,
    hsm_name_available_vec: Vec<String>,
) -> Result<(), backend_dispatcher::error::Error> {
    for target_hsm_name in target_hsm_name_vec {
        if !hsm_name_available_vec.contains(&target_hsm_name) {
            let err_msg = format!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_name,
                hsm_name_available_vec.join(", ")
            );

            return Err(backend_dispatcher::error::Error::Message(err_msg));
        }
    }

    Ok(())
}

/// Returns a list of HSM groups the user is expected to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// If the user did not requested any HSM group, then it will return all HSM
/// groups he has access to
/// hsm_group_cli_arg_opt: may contain a comma separated list of HSM groups defined in CLI command
/// arguments
/// hsm_group_env_or_config_file_opt: may contain a comma separated list of HSM groups defined in
/// either environment variable or configuration file
pub async fn get_target_hsm_name_group_vec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_cli_arg_opt: Option<&String>,
    hsm_group_env_or_config_file_opt: Option<&String>,
) -> Result<Vec<String>, backend_dispatcher::error::Error> {
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    let target_hsm_name_vec = if let Some(hsm_group_cli_arg) = hsm_group_cli_arg_opt {
        hsm_group_cli_arg
            .split(",")
            .map(|hsm_group| hsm_group.trim().to_string())
            .collect()
    } else if let Some(hsm_group_env_or_config_file) = hsm_group_env_or_config_file_opt {
        hsm_group_env_or_config_file
            .split(",")
            .map(|hsm_group| hsm_group.trim().to_string())
            .collect()
    } else {
        hsm_name_available_vec.clone()
    };

    let _ = validate_hsm_groups(&target_hsm_name_vec, hsm_name_available_vec);

    Ok(target_hsm_name_vec.clone())
} */
