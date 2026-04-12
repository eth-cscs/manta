use std::collections::HashMap;

use anyhow::{Error, bail};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::common::{
  self, app_context::AppContext, audit, authentication::get_api_token,
};

/// Move nodes between HSM groups with validation.
pub async fn exec(
  ctx: &AppContext<'_>,
  target_hsm_name_vec: &[String],
  parent_hsm_name_vec: &[String],
  hosts_expression: &str,
  dry_run: bool,
  create_hsm_group: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let kafka_audit_opt = ctx.kafka_audit_opt;

  let shasta_token = get_api_token(backend, site_name).await?;

  // Filter xnames to the ones members to HSM groups the user has access to
  //
  // Convert user input to xname
  let xname_to_move_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
    hosts_expression,
    false,
  )
  .await?;

  if xname_to_move_vec.is_empty() {
    bail!(
      "The list of nodes to operate is empty. \
       Nothing to do",
    );
  }

  // Get HashMap with HSM groups and members curated for this request.
  // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
  // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
  // hostlist have been removed
  let mut hsm_group_summary: HashMap<String, Vec<String>> =
    common::node_ops::get_curated_hsm_group_from_xname_hostlist(
      backend,
      &shasta_token,
      &xname_to_move_vec,
    )
    .await?;

  // Keep HSM groups based on list of parent HSM groups provided
  hsm_group_summary
    .retain(|hsm_name, _xname_vec| parent_hsm_name_vec.contains(hsm_name));

  log::debug!("xnames to move: {:?}", xname_to_move_vec);

  for target_hsm_name in target_hsm_name_vec {
    if backend
      .get_group(&shasta_token, target_hsm_name)
      .await
      .is_ok()
    {
      log::debug!("The HSM group {} exists, good.", target_hsm_name);
    } else if create_hsm_group {
      log::info!(
        "HSM group {} does not exist, it will be created",
        target_hsm_name
      );
      if !dry_run {
      } else {
        bail!(
          "Dry-run selected, cannot create the \
           new group continue.",
        );
      }
    } else {
      bail!(
        "HSM group {} does not exist, but the option \
         to create the group was NOT specified, \
         cannot continue.",
        target_hsm_name
      );
    }

    // Migrate nodes
    for (parent_hsm_name, xname_to_move_vec) in &hsm_group_summary {
      let node_migration_rslt = backend
        .migrate_group_members(
          &shasta_token,
          target_hsm_name,
          parent_hsm_name,
          &xname_to_move_vec
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>(),
        )
        .await;

      match node_migration_rslt {
        Ok((
          mut target_hsm_group_member_vec,
          mut parent_hsm_group_member_vec,
        )) => {
          target_hsm_group_member_vec.sort();
          parent_hsm_group_member_vec.sort();
          println!(
            "HSM '{}' members: {:?}",
            target_hsm_name, target_hsm_group_member_vec
          );
          println!(
            "HSM '{}' members: {:?}",
            parent_hsm_name, parent_hsm_group_member_vec
          );
        }
        Err(e) => return Err(e.into()),
      }
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    audit::send_audit(
      kafka_audit,
      &shasta_token,
      format!(
        "Migrate nodes from {:?} to {:?}",
        parent_hsm_name_vec, target_hsm_name_vec
      ),
      Some(serde_json::json!(xname_to_move_vec)),
      Some(serde_json::json!(vec![
        parent_hsm_name_vec,
        target_hsm_name_vec
      ])),
    )
    .await;
  }

  Ok(())
}
