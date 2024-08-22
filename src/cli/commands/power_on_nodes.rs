use mesa::common::jwt_ops::get_claims_from_jwt_token;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
) {
    let operation = "on";

    let _ = mesa::pcs::transitions::http_client::post(
        shasta_base_url,
        shasta_token,
        shasta_root_cert,
        operation,
        &xname_vec,
    )
    .await;

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);

    /* // Check Nodes are shutdown
    let _ = capmc::http_client::node_power_status::post(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec,
    )
    .await
    .unwrap();

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);

    let _ = wait_nodes_to_power_on(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname_vec,
        reason_opt,
    )
    .await; */
}
