use mesa::node::utils::validate_xnames;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    xname_string: &str,
) {
    let new_target_hsm_members = xname_string
        .split(",")
        .map(|xname| xname.trim())
        .collect::<Vec<&str>>();

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
            .split(",")
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
}
