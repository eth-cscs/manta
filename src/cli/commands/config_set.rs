use std::{fs, io::Write, path::PathBuf};

use directories::ProjectDirs;
use toml_edit::{value, Document};

pub async fn exec(shasta_token: &str, shasta_base_url: &str, new_hsm_opt: Option<&String>) {
    // Read configuration file

    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path_to_manta_configuration_file = PathBuf::from(project_dirs.unwrap().config_dir());

    path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

    log::debug!(
        "Reading manta configuration from {}",
        &path_to_manta_configuration_file.to_string_lossy()
    );

    let config_file_content = fs::read_to_string(path_to_manta_configuration_file.clone())
        .expect("Error reading configuration file");

    let mut doc = config_file_content
        .parse::<Document>()
        .expect("ERROR: could not parse configuration file to TOML");

    // VALIDATION
    let hsm_available_vec;
    if doc.get("hsm_available").is_some()
        && doc["hsm_available"].as_array().is_some()
        && !doc["hsm_available"].as_array().unwrap().is_empty()
    {
        // If hsm_available config param has values, then a tenant is running manta ==> enfore
        // config param 'hsm_group' has a value from 'hsm_available' because tenants can't unset
        // 'hsm_group' otherwise they will be able to operate on any HSM group in the system.
        // Note: tenants can't modify the configuration file directly because of manta runs as
        // manta user using sticky bit
        hsm_available_vec = doc["hsm_available"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hsm_group_value| hsm_group_value.as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        /* if new_hsm_opt.is_none() {
            println!("new hsm is empty!");
            eprintln!(
                "Please provide one of the following HSM values {:?}",
                hsm_available_vec
            );
            std::process::exit(1);
        } */

        validate_hsm_group_and_hsm_available_config_params(new_hsm_opt.unwrap(), hsm_available_vec);

        // All goot, we are safe to update 'hsm_group' config param
        log::info!(
            "Changing configuration to use HSM GROUP {}",
            new_hsm_opt.unwrap()
        );

        doc["hsm_group"] = value(new_hsm_opt.unwrap());
    } else if new_hsm_opt.is_none() {
        // 'hsm_available' config param is empty or does not exists, then an admin user is running
        // manta and 'hsm_group' config param is empty or does not exists, then it is safe to remove
        // this param from the config file
        log::info!("New HSM value not provided. Unset 'hsm_group' config param");
        doc.remove("hsm_group");
    } else {
        // 'hsm_available' config param is empty or does not exists (an admin user is running manta)
        // and 'hsm_group' has a value, then we fetch all HSM groups from CSM and check the user is
        // asking to put a valid HSM group in the configuration file
        hsm_available_vec =
            mesa::shasta::hsm::http_client::get_all_hsm_groups(shasta_token, shasta_base_url)
                .await
                .unwrap()
                .into_iter()
                .map(|hsm_group_value| hsm_group_value["label"].as_str().unwrap().to_string())
                .collect::<Vec<String>>();

        validate_hsm_group_and_hsm_available_config_params(new_hsm_opt.unwrap(), hsm_available_vec);

        // All goot, we are safe to update 'hsm_group' config param
        log::info!(
            "Changing configuration to use HSM GROUP {}",
            new_hsm_opt.unwrap()
        );

        doc["hsm_group"] = value(new_hsm_opt.unwrap());
    };

    // Update configuration file content
    let mut manta_configuration_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path_to_manta_configuration_file)
        .unwrap();

    /* let mut output = File::create(path_to_manta_configuration_file).unwrap();
    write!(output, "{}", doc.to_string()); */

    manta_configuration_file
        .write_all(doc.to_string().as_bytes())
        .unwrap();
    manta_configuration_file.flush().unwrap();

    match doc.get("hsm_group") {
        Some(hsm_value) => println!("Target HSM group set to {hsm_value}"),
        None => println!("Target HSM group unset"),
    }
}

pub fn validate_hsm_group_and_hsm_available_config_params(
    hsm_group: &String,
    hsm_available_vec: Vec<String>,
) {
    if !hsm_available_vec.contains(hsm_group) {
        eprintln!(
            "HSM group provided ({}) not valid, please choose one of the following options: {:?}",
            hsm_group, hsm_available_vec
        );
        std::process::exit(1);
    }
}

/* pub fn unset_hsm(new_hsm_opt: Option<&String>, hsm_available: Option<&toml_edit::Item>) {
    if hsm_available.is_some()
        && hsm_available.unwrap().as_array().is_some()
        && !hsm_available.unwrap().as_array().unwrap().is_empty()
    {
        println!("HSM can't be unset. Exit");
        std::process::exit(1);
    }
} */
