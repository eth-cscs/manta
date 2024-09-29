pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    xname_to_move_string: &str,
    nodryrun: bool,
    create_hsm_group: bool,
) {
    let xname_to_move_vec = xname_to_move_string
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

    let node_migration_rslt = mesa::hsm::group::utils::migrate_hsm_members(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_hsm_group_name,
        parent_hsm_group_name,
        xname_to_move_vec,
        nodryrun,
    )
    .await;

    match node_migration_rslt {
        Ok((target_hsm_group_member_vec, parent_hsm_group_member_vec)) => {
            println!(
                "HSM '{}' members: {:?}",
                target_hsm_group_name, target_hsm_group_member_vec
            );
            println!(
                "HSM '{}' members: {:?}",
                parent_hsm_group_name, parent_hsm_group_member_vec
            );
        }
        Err(e) => eprintln!("{}", e),
    }
}
