pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
) {
    let _ = mesa::capmc::http_client::node_power_off::post_sync(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname_vec,
        reason_opt,
        force,
    )
    .await;
}
