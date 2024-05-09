use mesa::{hsm, node::utils::validate_xnames};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    xname_string: &str,
    nodryrun: bool,
    create_hsm_group: bool,
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
            if create_hsm_group {
                log::info!("HSM group {} does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
                if nodryrun {
                    mesa::hsm::group::mesa::http_client::create_new_hsm_group(shasta_token, shasta_base_url, shasta_root_cert, target_hsm_group_name, &[], "false", "", &[])
                        .await.expect("Unable to create new HSM group");
                } else {
                    log::error!("Dryrun selected, cannot create the new group continue.");
                    std::process::exit(1);
                }
            }
            else {
                log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
                std::process::exit(1);
            }
        }
    };

    if !validate_xnames(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_target_hsm_members.as_slice(),
        Some(&parent_hsm_group_name.to_string()),
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
    target_hsm_group_member_vec.extend(
        xname_string
            .split(',')
            .map(|xname| xname.trim().to_string())
            .collect::<Vec<String>>(),
    );

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
        .retain(|parent_member| !target_hsm_group_member_vec.contains(parent_member));

    parent_hsm_group_member_vec.sort();
    parent_hsm_group_member_vec.dedup();

    let target_hsm_group = serde_json::json!({
        "label": target_hsm_group_name,
        "decription": "",
        "members": target_hsm_group_member_vec,
        "tags": []
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&target_hsm_group).unwrap()
    );

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
        log::info!("Dryrun enabled, not modifying the HSM groups on the system.")
    } else {
        for xname in new_target_hsm_members {
            // TODO: This is creating a new client per xname, look whether this can be improved reusing the client.
            let _ = hsm::group::shasta::http_client::post_member(shasta_token, shasta_base_url, shasta_root_cert, target_hsm_group_name, xname).await;

            let _ = hsm::group::shasta::http_client::delete_member(shasta_token, shasta_base_url, shasta_root_cert, parent_hsm_group_name, xname).await;
        }
    }
}
