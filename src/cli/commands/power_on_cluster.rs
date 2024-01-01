pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_arg_opt: Option<&String>,
) {
    let xname_vec = mesa::hsm::group::shasta::utils::get_members_ids(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name_arg_opt.unwrap(),
    )
    .await;

    let _ = mesa::capmc::http_client::node_power_on::post_sync(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname_vec,
        None,
    )
    .await;
}
