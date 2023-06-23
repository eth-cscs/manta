use mesa::shasta::bos;


pub async fn exec(
    // hsm_group: Option<&String>,
    // cli_get_template: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    template_name: Option<&String>,
    most_recent: Option<bool>,
    limit: Option<&u8>,
) {
    let limit_number;

    // let template_name = cli_get_template.get_one::<String>("name");

    /* let hsm_group_name = match hsm_group {
        None => cli_get_template.get_one::<String>("hsm-group"),
        Some(hsm_group_val) => Some(hsm_group_val),
    }; */

    // let most_recent = cli_get_template.get_one::<bool>("most-recent");

    if let Some(true) = most_recent {
        limit_number = Some(&1);
    } else if let Some(false) = most_recent {
        limit_number = limit;
    } else {
        limit_number = None;
    }

    let bos_templates = bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        template_name,
        limit_number,
    )
    .await
    .unwrap_or_default();

    if bos_templates.is_empty() {
        println!("No BOS template found!");
        std::process::exit(0);
    } else {
        bos::template::utils::print_table(bos_templates);
    }
}
