use anyhow::Error;
use backend_dispatcher::types::{
    bos::session_template::BosSessionTemplate,
    cfs::cfs_configuration_details::ConfigurationDetails,
    cfs::cfs_configuration_response::CfsConfigurationResponse, cfs::session::CfsSessionGetResponse,
    ims::Image,
};
use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::Table;
use globset::Glob;
use mesa::cfs::configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse as MesaCfsConfigurationResponse;

pub fn print_table_struct(cfs_configurations: &Vec<CfsConfigurationResponse>) {
    let mut table = Table::new();

    table.set_header(vec!["Config Name", "Last updated", "Layers"]);

    for cfs_configuration in cfs_configurations {
        let mut layers: String = String::new();

        if !cfs_configuration.layers.is_empty() {
            let layers_json = &cfs_configuration.layers;

            layers = format!(
                "Name:     {}\nPlaybook: {}\nCommit:   {}",
                layers_json[0].name,
                layers_json[0].playbook,
                layers_json[0]
                    .commit
                    .as_ref()
                    .unwrap_or(&"Not defined".to_string()),
            );

            for layer in layers_json.iter().skip(1) {
                layers = format!(
                    "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
                    layers,
                    layer.name,
                    layer.playbook,
                    layer.commit.as_ref().unwrap_or(&"Not defined".to_string()),
                );
            }
        }

        table.add_row(vec![
            cfs_configuration.name.clone(),
            cfs_configuration
                .last_updated
                .clone()
                .parse::<DateTime<Local>>()
                .unwrap()
                .format("%d/%m/%Y %H:%M:%S")
                .to_string(),
            layers,
        ]);
    }

    println!("{table}");
}

pub fn print_table_details_struct(
    cfs_configuration: ConfigurationDetails,
    cfs_session_vec_opt: Option<Vec<CfsSessionGetResponse>>,
    bos_sessiontemplate_vec_opt: Option<Vec<BosSessionTemplate>>,
    image_vec_opt: Option<Vec<Image>>,
) {
    let mut table = Table::new();

    table.set_header(vec![
        "Configuration Name",
        "Last updated",
        "Layers",
        "Derivatives",
    ]);

    let mut layers: String = String::new();

    for layer in cfs_configuration.config_layers {
        layers = format!(
            "{}\n\nName:     {}\nBranch:   {}\nTag:      {}\nDate:     {}\nAuthor:   {}\nCommit:   {}\nPlaybook: {}",
            layers,
            layer.name,
            layer.branch,
            /* if let true = layer.most_recent_commit {
                "(Up to date)"
            } else {
                "(Outdated)"
            }, */
            layer.tag,
            layer.commit_date,
            layer.author,
            layer.commit_id,
            layer.playbook
        );
    }

    let mut derivatives: String = String::new();

    if let Some(cfs_session_vec) = cfs_session_vec_opt {
        derivatives = derivatives + "CFS sessions:";
        for cfs_session in cfs_session_vec {
            derivatives = derivatives + "\n - " + &cfs_session.name.unwrap();
        }
    }

    if let Some(bos_sessiontemplate_vec) = bos_sessiontemplate_vec_opt {
        derivatives = derivatives + "\n\nBOS sessiontemplates:";
        for bos_sessiontemplate in bos_sessiontemplate_vec {
            derivatives = derivatives + "\n - " + &bos_sessiontemplate.name.unwrap();
        }
    }

    if let Some(image_vec) = image_vec_opt {
        derivatives = derivatives + "\n\nIMS images:";
        for image in image_vec {
            derivatives = derivatives + "\n - " + &image.name;
        }
    }

    layers = layers.trim_start_matches("\n\n").to_string();

    table.add_row(vec![
        cfs_configuration.name,
        cfs_configuration
            .last_updated
            .parse::<DateTime<Local>>()
            .unwrap()
            .format("%d/%m/%Y %H:%M:%S")
            .to_string(),
        layers,
        derivatives,
    ]);

    println!("{table}");
}

pub fn filter(
    cfs_configuration_vec: &mut Vec<CfsConfigurationResponse>,
    configuration_name_pattern_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::info!("Filter CFS configurations");

    // Filter CFS configurations based on user input (date range or configuration name)
    if let (Some(since), Some(until)) = (since_opt, until_opt) {
        cfs_configuration_vec.retain(|cfs_configuration| {
            let date = chrono::DateTime::parse_from_rfc3339(&cfs_configuration.last_updated)
                .unwrap()
                .naive_utc();

            since <= date && date < until
        });
    }

    // Sort by last updated date in ASC order
    cfs_configuration_vec.sort_by(|cfs_configuration_1, cfs_configuration_2| {
        cfs_configuration_1
            .last_updated
            .cmp(&cfs_configuration_2.last_updated)
    });

    if let Some(limit_number) = limit_number_opt {
        // Limiting the number of results to return to client

        *cfs_configuration_vec = cfs_configuration_vec[cfs_configuration_vec
            .len()
            .saturating_sub(*limit_number as usize)..]
            .to_vec();
    }

    // Filter CFS configurations based on pattern matching
    if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
        let glob = Glob::new(configuration_name_pattern)
            .unwrap()
            .compile_matcher();
        cfs_configuration_vec
            .retain(|cfs_configuration| glob.is_match(cfs_configuration.name.clone()));
    }

    Ok(cfs_configuration_vec.to_vec())
}
