use dialoguer::theme::ColorfulTheme;
use mesa::common::jwt_ops;

use crate::common::{audit::Audit, kafka::Kafka};

pub async fn exec(
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    label: &str,
    assume_yes: bool,
    dryrun: bool,
    kafka_audit: &Kafka,
) {
    // Validate if group can be deleted
    validation(auth_token, base_url, root_cert, label).await;

    if !assume_yes {
        let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will delete the group '{}'.\nPlease confirm to proceed",
                label
            ))
            .interact()
            .unwrap();

        if !proceed {
            println!("Operation canceled by the user. Exit");
            std::process::exit(1);
        }
    }

    if dryrun {
        println!("Dryrun mode: group '{}' would be deleted", label);
        return;
    }

    // Delete group
    let result =
        mesa::hsm::group::http_client::delete(auth_token, base_url, root_cert, &label.to_string())
            .await;

    match result {
        Ok(_) => {
            println!("Group '{}' deleted", label);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    // Audit
    let username = jwt_ops::get_name(auth_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(auth_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": Vec::<String>::new()}, "message": format!("Delete Group '{}'", label)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}

// Checks if a group can be deleted.
// A group can be deleted if none of its members becomes orphan.
// A group member is orphan if it does not have a group assigned to it
async fn validation(auth_token: &str, base_url: &str, root_cert: &[u8], label: &str) {
    // Find the list of xnames belonging only to the label to delete and if any, then stop
    // processing the request because those nodes can't get orphan
    let xname_vec = mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
        auth_token,
        base_url,
        root_cert,
        vec![label.to_string()],
    )
    .await;

    let xname_vec = xname_vec.iter().map(|e| e.as_str()).collect();

    let mut xname_map =
        mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_without_system_wide_vec(
            auth_token, base_url, root_cert, xname_vec,
        )
        .await
        .unwrap();

    xname_map.retain(|_xname, group_name_vec| {
        group_name_vec.len() == 1 && group_name_vec.first().unwrap() == label
    });

    let mut members_orphan_if_group_deleted: Vec<String> = xname_map
        .into_iter()
        .map(|(xname, _)| xname.clone())
        .collect();

    members_orphan_if_group_deleted.sort();

    /* println!(
        "DEBUG - members orphan if group deleted:\n{:#?}",
        members_orphan_if_group_deleted
    ); */

    if !members_orphan_if_group_deleted.is_empty() {
        eprintln!(
            "ERROR - The hosts below will become orphan if group '{}' gets deleted.\n{:?}\n",
            label, members_orphan_if_group_deleted
        );
        std::process::exit(1);
    }
}
