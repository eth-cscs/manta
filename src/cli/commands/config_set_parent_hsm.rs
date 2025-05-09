use std::{fs, io::Write, path::PathBuf};

use directories::ProjectDirs;
use toml_edit::{value, Document};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    new_hsm_opt: Option<&String>,
    // all_hsm_available_vec: &[String],
) {
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

    let mut settings_hsm_available_vec =
        csm_rs::common::jwt_ops::get_roles(shasta_token).unwrap_or(Vec::new());

    settings_hsm_available_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    // VALIDATION
    let hsm_available_vec = if settings_hsm_available_vec.is_empty() {
        csm_rs::hsm::group::http_client::get_all_without_system_wide(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await
        .unwrap()
        .into_iter()
        .map(|hsm_group_value| hsm_group_value.label)
        .collect::<Vec<String>>()
    } else {
        settings_hsm_available_vec
    };

    validate_hsm_group_and_hsm_available_config_params(new_hsm_opt.unwrap(), &hsm_available_vec);

    // All goot, we are safe to update 'hsm_group' config param
    log::info!(
        "Changing configuration to use HSM GROUP {}",
        new_hsm_opt.unwrap()
    );

    doc["parent_hsm_group"] = value(new_hsm_opt.unwrap());

    if let Some(new_hsm) = new_hsm_opt {
        // 'hsm_available' config param is empty or does not exists (an admin user is running manta)
        // and 'hsm_group' has a value, then we fetch all HSM groups from CSM and check the user is
        // asking to put a valid HSM group in the configuration file
        let all_hsm_available_vec = csm_rs::hsm::group::http_client::get_all_without_system_wide(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await
        .unwrap()
        .into_iter()
        .map(|hsm_group_value| hsm_group_value.label)
        .collect::<Vec<String>>();

        validate_hsm_group_and_hsm_available_config_params(
            new_hsm_opt.unwrap(),
            &all_hsm_available_vec,
        );

        // All goot, we are safe to update 'hsm_group' config param
        log::info!(
            "Changing configuration to use HSM GROUP {}",
            new_hsm_opt.unwrap()
        );

        doc["parent_hsm_group"] = value(new_hsm);
    } else {
        // 'hsm_available' config param is empty or does not exists, then an admin user is running
        // manta and 'hsm_group' config param is empty or does not exists, then it is safe to remove
        // this param from the config file
        //
        // NOTHING TO DO
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

    match doc.get("parent_hsm_group") {
        Some(hsm_value) => println!("Parent HSM group set to {hsm_value}"),
        None => println!("Parent HSM group unset"),
    }
}

pub fn validate_hsm_group_and_hsm_available_config_params(
    hsm_group: &String,
    hsm_available_vec: &[String],
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
