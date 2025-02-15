use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{
        audit::Audit, authorization::validate_target_hsm_members, kafka::Kafka,
        node_ops::resolve_node_list_user_input_to_xname,
    },
};
use backend_dispatcher::{interfaces::hsm::group::GroupTrait, types::Group};
use dialoguer::theme::ColorfulTheme;
use mesa::common::jwt_ops;

pub async fn exec(
    backend: StaticBackendDispatcher,
    auth_token: &str,
    label: &str,
    node_expression: Option<&String>,
    assume_yes: bool,
    is_regex: bool,
    dryrun: bool,
    kafka_audit: &Kafka,
) {
    let xname_vec_opt: Option<Vec<String>> = match node_expression {
        Some(node_expression) => {
            let xname_vec: Vec<String> = resolve_node_list_user_input_to_xname(
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
    };

    // Validate user has access to the list of xnames requested
    if let Some(xname_vec) = &xname_vec_opt {
        validate_target_hsm_members(&backend, &auth_token, xname_vec).await;
    }

    // Create Group instance for http payload
    let group = Group::new(label, xname_vec_opt.clone(), None, None);

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
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    // Audit
    let username = jwt_ops::get_name(auth_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(auth_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec_opt.unwrap_or_default()}, "message": format!("Create Group '{}'", label)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
}
