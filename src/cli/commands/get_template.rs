pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &Vec<String>,
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
) {
    log::info!(
        "Get BOS sessiontemplates for HSM groups: {:?}",
        hsm_group_name_vec
    );

    let mut bos_sessiontemplate_vec = mesa::bos::template::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos_sessiontemplate_name_opt,
    )
    .await
    .unwrap_or_default();

    bos_sessiontemplate_vec = mesa::bos::template::mesa::utils::filter(
        &mut bos_sessiontemplate_vec,
        hsm_group_name_vec,
        hsm_member_vec,
        None,
        limit_number_opt,
    )
    .await;

    if bos_sessiontemplate_vec.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        crate::common::bos_sessiontemplate_utils::print_table_struct(bos_sessiontemplate_vec);
    }
}
