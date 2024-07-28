use mesa::{
    capmc::http_client::node_power_reset::post_sync, common::jwt_ops::get_claims_from_jwt_token,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
) {
    /* let mut nodes_reseted = Vec::new();

    let mut tasks = tokio::task::JoinSet::new();

    for xname in xname_vec.clone() {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();
        let reason_cloned = reason_opt.clone();

        tasks.spawn(async move {
            post_sync(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                vec![xname.to_string()],
                reason_cloned,
                force,
            )
            .await
            .unwrap()
        });
    }

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power reset nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);

    while let Some(message) = tasks.join_next().await {
        if let Ok(node_power_status) = message {
            nodes_reseted.push(node_power_status);
        }
    } */

    post_sync(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname_vec.clone(),
        reason_opt,
        force,
    )
    .await
    .unwrap();

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power reset nodes {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec);
}
