use clap::ArgMatches;

use crate::shasta::cfs::session as shasta_cfs_session;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_get_session: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
) {
    let session_name = cli_get_session.get_one::<String>("name");

    let hsm_group_name = match hsm_group {
        // ref: https://stackoverflow.com/a/32487173/1918003
        None => cli_get_session.get_one::<String>("hsm-group"),
        Some(hsm_group_val) => Some(hsm_group_val),
    };

    let most_recent = cli_get_session.get_one::<bool>("most-recent");

    let limit_number;

    if let Some(true) = most_recent {
        limit_number = Some(&1);
    } else if let Some(false) = most_recent {
        limit_number = cli_get_session.get_one::<u8>("limit");
    } else {
        limit_number = None;
    }

    let cfs_sessions = shasta_cfs_session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        session_name,
        limit_number,
        None,
    )
    .await
    .unwrap_or_default();

    log::info!("CFS sessions:\n{:#?}", cfs_sessions);

    if cfs_sessions.is_empty() {
        println!("CFS session not found!");
        std::process::exit(0);
    } else {
        shasta_cfs_session::utils::print_table(cfs_sessions);
    }
}
