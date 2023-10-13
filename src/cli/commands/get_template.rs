use mesa::{mesa::bos::sessiontemplate::utils::print_table_struct, shasta::bos};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_opt: Option<&String>,
    bos_sessiontemplate_name_opt: Option<&String>,
    most_recent_opt: Option<bool>,
    limit_number_opt: Option<&u8>,
) {
    let mut bos_sessiontemplate_vec = mesa::mesa::bos::sessiontemplate::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        /* hsm_group_name,
        template_name,
        limit_number, */
    )
    .await
    .unwrap_or_default();

    bos_sessiontemplate_vec = mesa::mesa::bos::sessiontemplate::utils::filter(
        &mut bos_sessiontemplate_vec,
        hsm_group_name_opt,
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
