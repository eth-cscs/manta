use anyhow::{Context, bail};

use crate::{
  common::{audit, authentication::get_api_token, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  label: &str,
  force: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), anyhow::Error> {
  let auth_token = get_api_token(backend, site_name).await?;
  if !force {
    // Validate if group can be deleted
    validation(backend, &auth_token, label).await?;
  }

  // Delete group
  backend
    .delete_group(&auth_token, label)
    .await
    .with_context(|| format!("Could not delete group '{}'", label))?;

  println!("Group '{}' deleted", label);

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(&auth_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(&auth_token).unwrap_or_default();

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "group": label,
      "message": format!(
        "Delete Group '{}'",
        label
      ),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

  Ok(())
}

// Checks if a group can be deleted.
// A group can be deleted if none of its members becomes orphan.
// A group member is orphan if it does not have a group assigned to it
async fn validation(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  label: &str,
) -> Result<(), anyhow::Error> {
  // Find the list of xnames belonging only to the label to delete and if any, then stop
  // processing the request because those nodes can't get orphan
  let xname_vec = backend
    .get_member_vec_from_group_name_vec(auth_token, &[label.to_string()])
    .await?;

  let xname_vec: Vec<&str> = xname_vec.iter().map(String::as_str).collect();

  let mut xname_map = backend
    .get_group_map_and_filter_by_group_vec(auth_token, &xname_vec)
    .await?;

  xname_map.retain(|_xname, group_name_vec| {
    group_name_vec.len() == 1
      && group_name_vec.first().is_some_and(|name| name == label)
  });

  let mut members_orphan_if_group_deleted: Vec<String> =
    xname_map.into_keys().collect();

  members_orphan_if_group_deleted.sort();

  if !members_orphan_if_group_deleted.is_empty() {
    bail!(
      "The hosts below will become orphan if group '{}' \
       gets deleted.\n{:?}",
      label,
      members_orphan_if_group_deleted
    );
  }

  Ok(())
}
