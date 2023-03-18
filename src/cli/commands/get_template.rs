use clap::ArgMatches;

use crate::shasta::bos::template as bos_template;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_get_template: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) {
    let limit_number;

    let template_name = cli_get_template.get_one::<String>("name");

    let hsm_group_name = match hsm_group {
        None => cli_get_template.get_one::<String>("hsm-group"),
        Some(hsm_group_val) => Some(hsm_group_val),
    };

    let most_recent = cli_get_template.get_one::<bool>("most-recent");

    if let Some(true) = most_recent {
        limit_number = Some(&1);
    } else if let Some(false) = most_recent {
        limit_number = cli_get_template.get_one::<u8>("limit");
    } else {
        limit_number = None;
    }

    let bos_templates = bos_template::http_client::get(
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
        bos_template::utils::print_table(bos_templates);
    }
}
