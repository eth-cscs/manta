pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: &str,
    xname_string: &String,
) {
    // get list of HSM group members
    let mut hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name,
        )
        .await;

    // merge HSM group list with the list of xnames provided by the user
    hsm_group_member_vec.retain(|xname| !xname_string.contains(xname));

    // submit to CSM api new list of HSM members
    log::info!(
        "HSM '{}' members: {:?}",
        hsm_group_name,
        hsm_group_member_vec
    );
}
