use k8s_openapi::url::form_urlencoded::Target;
use mesa::{hsm, node::utils::validate_xnames};
use mesa::hsm::group::mesa::http_client::delete_hsm_group;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    xname_string: &str,
    nodryrun: bool,
    delete_hsm_group: bool,
) {


    let new_target_hsm_members = xname_string
        .split(',')
        .map(|xname| xname.trim())
        .collect::<Vec<&str>>();

    match mesa::hsm::group::mesa::http_client::get(shasta_token,
                                                   shasta_base_url,
                                                   shasta_root_cert,
                                                   Some(&target_hsm_group_name.to_string())).await {
        Ok(_) => log::debug!("The HSM group {} exists, good.",target_hsm_group_name),
        Err(error) => {
            log::error!("HSM group {} does not exist, cannot remove xnames from it and cannot continue.", target_hsm_group_name.to_string());
            std::process::exit(1);
        }
    }
    if !validate_xnames(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_target_hsm_members.as_slice(),
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    {
        eprintln!("Nodes '{:?}' not valid", new_target_hsm_members);
        std::process::exit(1);
    }

    // get list of target HSM group members
    let mut target_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_group_name,
        )
        .await;

    // merge HSM group list with the list of xnames provided by the user
    target_hsm_group_member_vec.retain(|xname| !xname_string.contains(xname));

    target_hsm_group_member_vec.sort();
    target_hsm_group_member_vec.dedup();

    // get list of parent HSM group members
    let mut parent_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            parent_hsm_group_name,
        )
        .await;

    parent_hsm_group_member_vec
        .extend(new_target_hsm_members.iter().map(|xname| xname.to_string()));

    parent_hsm_group_member_vec.sort();
    parent_hsm_group_member_vec.dedup();

    let hsm_group = serde_json::json!({
        "label": target_hsm_group_name,
        "decription": "",
        "members": target_hsm_group_member_vec,
        "tags": []
    });

    println!("{}", serde_json::to_string_pretty(&hsm_group).unwrap());

    let parent_hsm_group = serde_json::json!({
        "label": parent_hsm_group_name,
        "decription": "",
        "members": parent_hsm_group_member_vec,
        "tags": []
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&parent_hsm_group).unwrap()
    );

    // *********************************************************************************************************
    // UPDATE HSM GROUP MEMBERS IN CSM
    if !nodryrun {
        log::info!("Dry run enabled, not modifying the HSM groups on the system.")
    } else {
        for xname in new_target_hsm_members {
            // TODO: This is creating a new client per xname, look whether this can be improved reusing the client.

            let _ = hsm::group::shasta::http_client::delete_member(shasta_token, shasta_base_url, shasta_root_cert, target_hsm_group_name, xname).await;

            let _ = hsm::group::shasta::http_client::post_member(shasta_token, shasta_base_url,shasta_root_cert, parent_hsm_group_name, xname).await;
        }
        if target_hsm_group_member_vec.is_empty() {
            if delete_hsm_group {
                log::info!("HSM group {} is now empty and the option to delete empty groups has been selected, removing it.",target_hsm_group_name);
                match hsm::group::mesa::http_client::delete_hsm_group(shasta_token,
                                                                shasta_base_url,
                                                                shasta_root_cert,
                                                                target_hsm_group_name.to_string().as_mut_string())
                    .await {
                    Ok(_) => log::info!("HSM group removed successfully."),
                    Err(e2) => log::debug!("Error removing the HSM group. This always fails, ignore please. Reported: {}", e2)
                };
            } else {
                log::debug!("HSM group {} is now empty and the option to delete empty groups has NOT been selected, will not remove it.",target_hsm_group_name)
            }

        }
    }
}
