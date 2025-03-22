use crate::common::{self, jwt_ops};
use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{audit::Audit, authorization::validate_target_hsm_members, kafka::Kafka},
};
use backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use backend_dispatcher::types::Component;
use backend_dispatcher::{interfaces::hsm::group::GroupTrait, types::Group};
use dialoguer::theme::ColorfulTheme;

pub async fn exec(
    backend: StaticBackendDispatcher,
    auth_token: &str,
    label: &str,
    description: Option<&String>,
    node_expression: Option<&String>,
    assume_yes: bool,
    dryrun: bool,
    kafka_audit_opt: Option<&Kafka>,
) {
    let xname_vec_opt: Option<Vec<String>> = match node_expression {
        Some(node_expression) => {
            // Convert user input to xname
            let xname_available_vec: Vec<String> = backend
                .get_group_available(auth_token)
                .await
                .unwrap_or_else(|e| {
                    eprintln!(
                        "ERROR - Could not get group list. Reason:\n{}",
                        e.to_string()
                    );
                    std::process::exit(1);
                })
                .iter()
                .flat_map(|group| group.get_members())
                .collect();

            let node_metadata_vec: Vec<Component> = backend
                .get_all_nodes(auth_token, Some("true"))
                .await
                .unwrap()
                .components
                .unwrap_or_default()
                .iter()
                .filter(|&node_metadata| {
                    xname_available_vec.contains(&node_metadata.id.as_ref().unwrap())
                })
                .cloned()
                .collect();

            let xname_vec = common::node_ops::resolve_node_list_user_input_to_xname_2(
                node_expression,
                false,
                node_metadata_vec,
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
                    e
                );
                std::process::exit(1);
            });

            Some(xname_vec)
        }
        None => None,
    };

    /* let xname_vec_opt: Option<Vec<String>> = match node_expression {
        Some(node_expression) => {
            let xname_vec: Vec<String> = resolve_node_list_user_input_to_xname(
                // let xname_vec: Vec<String> = resolve_node_list_user_input_to_xname_2(
                &backend,
                auth_token,
                node_expression,
                false,
                is_regex,
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!("ERROR - Could not resolve node list. Reason:\n{e}\nExit");
                std::process::exit(1);
            });
            Some(xname_vec)
        }
        None => None,
    }; */

    // Validate user has access to the list of xnames requested
    if let Some(xname_vec) = &xname_vec_opt {
        validate_target_hsm_members(&backend, &auth_token, xname_vec).await;
    }

    // Create Group instance for http payload
    let group = Group::new(
        label,
        description.cloned(),
        xname_vec_opt.clone(),
        None,
        None,
    );

    if !assume_yes {
        let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will create the group below:\n{}\nPlease confirm to proceed",
                serde_json::to_string_pretty(&group).unwrap()
            ))
            .interact()
            .unwrap();

        if !proceed {
            println!("Operation canceled by the user. Exit");
            std::process::exit(1);
        }
    }

    if dryrun {
        println!(
            "Dryrun mode: The group below would be created:\n{}",
            serde_json::to_string_pretty(&group).unwrap()
        );
        return;
    }

    // Call backend to create group
    let result = backend.add_group(&auth_token, group).await;

    match result {
        Ok(_) => {
            eprintln!("Group '{}' created", label);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(auth_token).unwrap_or_default();
        let user_id = jwt_ops::get_preferred_username(auth_token).unwrap_or_default();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec_opt.unwrap_or_default()}, "group": label, "message": format!("Create Group '{}'", label)});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
    }
}
