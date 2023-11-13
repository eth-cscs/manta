use mesa::mesa::bos::sessiontemplate::utils::print_table_struct;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &Vec<String>,
    hsm_member_vec: &Vec<String>,
    bos_sessiontemplate_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
) {
    log::info!("Get BOS sessiontemplates for HSM groups: {:?}", hsm_group_name_vec);

    let mut bos_sessiontemplate_vec = mesa::mesa::bos::sessiontemplate::http_client::get_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await
    .unwrap_or_default();

    bos_sessiontemplate_vec = mesa::mesa::bos::sessiontemplate::utils::filter(
        &mut bos_sessiontemplate_vec,
        hsm_group_name_vec,
        hsm_member_vec,
        bos_sessiontemplate_name_opt,
        limit_number_opt,
    )
    .await;

    if bos_sessiontemplate_vec.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        print_table_struct(bos_sessiontemplate_vec);
    }
}
