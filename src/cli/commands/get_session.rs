use mesa::shasta::cfs;

use mesa::manta;


pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    session_name: Option<&String>,
    limit_number: Option<&u8>,
) {
    let cfs_session_table_data_list = manta::cfs::session::get_sessions(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        session_name,
        limit_number,
    )
    .await;

    // println!("{:#?}", cfs_session_table_data_list);

    if cfs_session_table_data_list.is_empty() {
        println!("CFS session not found!");
        std::process::exit(0);
    } else {
        cfs::session::utils::print_table(cfs_session_table_data_list);
    }
}
