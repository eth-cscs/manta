use crate::common::{self, jwt_ops};
use crate::{
  common::{
    audit::Audit, authorization::validate_target_hsm_members, kafka::Kafka,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use anyhow::Error;
use dialoguer::theme::ColorfulTheme;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

/// Creates a group of nodes. It is allowed to create a group with no nodes.
pub async fn exec(
  backend: StaticBackendDispatcher,
  auth_token: &str,
  label: &str,
  description: Option<&String>,
  hosts_expression_opt: Option<&String>,
  assume_yes: bool,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let xname_vec_opt: Option<Vec<String>> = match hosts_expression_opt {
    Some(hosts_expression) => {
      // Convert user input to xname
      let node_metadata_available_vec = backend
        .get_node_metadata_available(auth_token)
        .await
        .map_err(|e| {
          Error::msg(format!(
            "ERROR - Could not get node metadata. Reason:\n{e}\nExit"
          ))
        })?;

      let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
        hosts_expression,
        false,
        node_metadata_available_vec,
      )
      .await
      .map_err(|e| {
        Error::msg(format!(
          "ERROR - Could not convert user input to list of xnames. Reason:\n{e}"
        ))
      })?;

      Some(xname_vec)
    }
    None => None,
  };

  // Validate user has access to the list of xnames requested
  if let Some(xname_vec) = &xname_vec_opt {
    validate_target_hsm_members(&backend, &auth_token, xname_vec).await?;
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
      return Err(Error::msg("Operation canceled by the user."));
    }
  }

  if dryrun {
    println!(
      "Dryrun mode: The group below would be created:\n{}",
      serde_json::to_string_pretty(&group).unwrap()
    );
    return Ok(());
  }

  // Call backend to create group
  let _ = backend.add_group(&auth_token, group).await?;

  eprintln!("Group '{}' created", label);

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(auth_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(auth_token).unwrap_or_default();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec_opt.unwrap_or_default()}, "group": label, "message": format!("Create Group '{}'", label)});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }

  Ok(())
}
