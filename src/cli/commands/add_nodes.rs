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

    if mesa::hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    .is_ok()
    {
        log::debug!("The HSM group {} exists, good.", target_hsm_group_name);
    } else {
        if create_hsm_group {
            log::info!("HSM group {} does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
            if nodryrun {
                mesa::hsm::group::http_client::create_new_hsm_group(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    &[],
                    "false",
                    "",
                    &[],
                )
                .await
                .expect("Unable to create new HSM group");
            } else {
                log::error!("Dry-run selected, cannot create the new group continue.");
                std::process::exit(1);
            }
        } else {
            log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
            std::process::exit(1);
        }
    }

    mesa::hsm::group::utils::migrate_hsm_members(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_hsm_group_name,
        parent_hsm_group_name,
        new_target_hsm_members,
        nodryrun,
    )
    .await;
}
