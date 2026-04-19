use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::Group;

use crate::common;
use crate::common::app_context::InfraContext;
use crate::common::authorization::{
  get_groups_names_available, validate_target_hsm_members,
};

/// Typed parameters for fetching HSM groups.
pub struct GetGroupParams {
  pub group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch HSM groups from the backend.
///
/// Resolves available group names, then fetches their details.
pub async fn get_groups(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetGroupParams,
) -> Result<Vec<Group>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.group_name.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let group_vec = infra.backend
    .get_groups(token, Some(&target_hsm_group_vec))
    .await
    .context("Failed to fetch HSM groups")?;

  Ok(group_vec)
}

/// Validate that deleting a group will not orphan any nodes.
///
/// A node is orphan if it belongs to no group after the deletion.
pub async fn validate_group_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  label: &str,
) -> Result<(), Error> {
  let xname_vec = infra
    .backend
    .get_member_vec_from_group_name_vec(token, &[label.to_string()])
    .await?;

  let xname_vec: Vec<&str> = xname_vec.iter().map(String::as_str).collect();

  let mut xname_map = infra
    .backend
    .get_group_map_and_filter_by_group_vec(token, &xname_vec)
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
       gets deleted.\n{}",
      label,
      members_orphan_if_group_deleted.join(", ")
    );
  }

  Ok(())
}

/// Delete an HSM group by label.
pub async fn delete_group(
  infra: &InfraContext<'_>,
  token: &str,
  label: &str,
  force: bool,
) -> Result<(), Error> {
  if !force {
    validate_group_deletion(infra, token, label).await?;
  }

  infra
    .backend
    .delete_group(token, label)
    .await
    .with_context(|| format!("Could not delete group '{}'", label))?;

  Ok(())
}

/// Resolve hosts and build a Group ready for creation.
///
/// Returns the constructed Group and the resolved xname list.
pub async fn prepare_add_group(
  infra: &InfraContext<'_>,
  token: &str,
  label: &str,
  description: Option<&str>,
  hosts_expression_opt: Option<&str>,
) -> Result<(Group, Option<Vec<String>>), Error> {
  let xname_vec_opt: Option<Vec<String>> = match hosts_expression_opt {
    Some(hosts_expression) => {
      let xname_vec = common::node_ops::resolve_hosts_expression(
        infra.backend,
        token,
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
    validate_target_hsm_members(infra.backend, token, xname_vec).await?;
  }

  let group = Group::new(
    label,
    description.map(str::to_string),
    xname_vec_opt.clone(),
    None,
    None,
  );

  Ok((group, xname_vec_opt))
}

/// Create an HSM group via the backend.
pub async fn create_group(
  infra: &InfraContext<'_>,
  token: &str,
  group: Group,
) -> Result<(), Error> {
  infra.backend.add_group(token, group).await?;
  Ok(())
}
