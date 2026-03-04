use anyhow::{Context, bail};
use manta_backend_dispatcher::{
  interfaces::{
    cfs::CfsTrait,
    hsm::{component::ComponentTrait, group::GroupTrait},
  },
  types::{Group, K8sDetails, cfs::session::CfsSessionGetResponse},
};

use crate::{
  common::{
    self, authentication::get_api_token,
    cfs_session_utils::check_cfs_session_against_groups_available,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use futures::{AsyncBufReadExt, TryStreamExt};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hosts_expression: &str,
  timestamps: bool,
  k8s: &K8sDetails,
) -> Result<(), anyhow::Error> {
  let shasta_token = get_api_token(backend, site_name).await?;
  let group_available_vec =
    backend.get_group_name_available(&shasta_token).await?;

  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .context("Could not get node metadata")?;

  let xname_vec_rslt = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await;

  // NOTE: fancy pattern matching. Maybe not a good use case for this. Ask in the future if this
  // is redeable or not
  let cfs_sessions_vec = match xname_vec_rslt.as_deref() {
    Ok([]) | Err(_) => {
      // Failed to convert user input to xname, try user input is either a group name or CFS session name
      log::debug!(
        "User input is not a node. Checking user input as CFS session name"
      );
      // Check if user input is a CFS session name
      backend
        .get_sessions(
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          Some(&hosts_expression.to_string()),
          None,
          None,
          None,
          None,
          None,
          None,
          None,
          None,
        )
        .await
    }
    Ok([xname]) => {
      // Get most recent CFS session for node or group the node belongs to
      log::debug!("User input is a single node");

      backend
        .get_and_filter_sessions(
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          group_available_vec.clone(),
          vec![xname],
          None,
          None,
          None,
          None,
          None,
          None,
          None,
        )
        .await
    }
    Ok([_, ..]) => {
      // User input is an expression that expands to multiple nodes
      log::debug!("User input is a list of nodes");
      bail!("Can only operate a single node");
    }
  }
  .context("Could not get CFS sessions")?;

  let cfs_session = cfs_sessions_vec.first().ok_or_else(|| {
    anyhow::anyhow!("No CFS session found for the given input")
  })?;

  log::info!(
    "Get logs for CFS session:\n{}",
    common::cfs_session_utils::get_table_struct(&cfs_sessions_vec)
  );

  let cfs_session_backend: CfsSessionGetResponse = cfs_session.clone();

  let group_available_vec_group = group_available_vec
    .iter()
    .map(|g| Group {
      label: g.clone(),
      description: None,
      tags: None,
      members: None,
      exclusive_group: None,
    })
    .collect();

  check_cfs_session_against_groups_available(
    &cfs_session_backend,
    group_available_vec_group,
  );

  print_cfs_session_logs(
    backend,
    &shasta_token,
    site_name,
    cfs_session.name.as_str(),
    timestamps,
    k8s,
  )
  .await?;

  Ok(())
}

pub async fn print_cfs_session_logs(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  site_name: &str,
  cfs_session_name: &str,
  timestamps: bool,
  k8s: &K8sDetails,
) -> Result<(), anyhow::Error> {
  let logs_stream = backend
    .get_session_logs_stream(
      shasta_token,
      site_name,
      cfs_session_name,
      timestamps,
      k8s,
    )
    .await?;

  let mut lines = logs_stream.lines();

  while let Some(line) =
    lines.try_next().await.context("Error reading log stream")?
  {
    println!("{}", line);
  }

  Ok(())
}
