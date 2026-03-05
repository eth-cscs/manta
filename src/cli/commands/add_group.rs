use crate::common::{self, audit, jwt_ops};
use crate::common::{
  app_context::AppContext, authorization::validate_target_hsm_members,
};
use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

/// Creates a group of nodes. It is allowed to create a group with no nodes.
pub async fn exec(
  ctx: &AppContext<'_>,
  auth_token: &str,
  label: &str,
  description: Option<&str>,
  hosts_expression_opt: Option<&str>,
  assume_yes: bool,
  dryrun: bool,
) -> Result<(), Error> {
  let backend = ctx.backend.clone();
  let kafka_audit_opt = ctx.kafka_audit_opt;
  let xname_vec_opt: Option<Vec<String>> = match hosts_expression_opt {
    Some(hosts_expression) => {
      // Convert user input to xname
      let xname_vec = common::node_ops::resolve_hosts_expression(
        &backend,
        auth_token,
        hosts_expression,
        false,
      )
      .await?;

      Some(xname_vec)
    }
    None => None,
  };

  // Validate user has access to the list of xnames requested
  if let Some(xname_vec) = &xname_vec_opt {
    validate_target_hsm_members(&backend, auth_token, xname_vec).await?;
  }

  // Create Group instance for http payload
  let group = Group::new(
    label,
    description.map(str::to_string),
    xname_vec_opt.clone(),
    None,
    None,
  );

  if !common::user_interaction::confirm(
    &format!(
      "This operation will create the group below:\n{}\nPlease confirm to proceed",
      serde_json::to_string_pretty(&group)
        .context("Failed to serialize group")?
    ),
    assume_yes,
  ) {
    bail!("Operation canceled by the user.");
  }

  if dryrun {
    println!(
      "Dryrun mode: The group below would be created:\n{}",
      serde_json::to_string_pretty(&group)
        .context("Failed to serialize group")?
    );
    return Ok(());
  }

  // Call backend to create group
  let _ = backend.add_group(auth_token, group).await?;

  println!("Group '{}' created", label);

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(auth_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(auth_token).unwrap_or_default();

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "host": {
        "hostname": xname_vec_opt.unwrap_or_default()
      },
      "group": label,
      "message": format!("Create Group '{}'", label),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

  Ok(())
}
