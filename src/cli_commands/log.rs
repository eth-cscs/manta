use clap::ArgMatches;

pub async fn exec(cli_log: &ArgMatches, shasta_token: &String, shasta_base_url: &String, vault_base_url: String) -> () {
    let logging_session_name = cli_log.get_one::<String>("SESSION");

    let layer_id = cli_log.get_one::<u8>("layer-id");

    crate::shasta_cfs_session_logs::client::session_logs_proxy(
        shasta_token,
        shasta_base_url,
        vault_base_url,
        None,
        logging_session_name,
        layer_id,
    )
    .await.unwrap();
}
