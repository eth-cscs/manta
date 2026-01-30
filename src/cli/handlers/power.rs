use crate::cli::commands::{
  power_off_cluster, power_off_nodes, power_on_cluster, power_on_nodes,
  power_reset_cluster, power_reset_nodes,
};
use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
  kafka::Kafka,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;

pub async fn handle_power(
  cli_power: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  settings_hsm_group_name_opt: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(cli_power_on) = cli_power.subcommand_matches("on") {
    if let Some(cli_power_on_cluster) =
      cli_power_on.subcommand_matches("cluster")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let hsm_group_name_arg = cli_power_on_cluster
        .get_one::<String>("CLUSTER_NAME")
        .expect("The 'cluster name' argument must have a value");
      let target_hsm_group_vec = get_groups_names_available(
        backend,
        &shasta_token,
        Some(hsm_group_name_arg),
        settings_hsm_group_name_opt,
      )
      .await?;
      let target_hsm_group = target_hsm_group_vec
        .first()
        .expect("The 'cluster name' argument must have a value");
      let assume_yes: bool = cli_power_on_cluster.get_flag("assume-yes");
      let output: &str =
        cli_power_on_cluster.get_one::<String>("output").unwrap();
      power_on_cluster::exec(
        backend.clone(), // StaticBackendDispatcher is usually cheap to clone or ref check
        &shasta_token,
        target_hsm_group,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    } else if let Some(cli_power_on_node) =
      cli_power_on.subcommand_matches("nodes")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let xname_requested: &str = cli_power_on_node
        .get_one::<String>("VALUE")
        .expect("The 'xnames' argument must have values");
      let assume_yes: bool = cli_power_on_node.get_flag("assume-yes");
      let output: &str = cli_power_on_node.get_one::<String>("output").unwrap();
      power_on_nodes::exec(
        backend,
        &shasta_token,
        xname_requested,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    }
  } else if let Some(cli_power_off) = cli_power.subcommand_matches("off") {
    if let Some(cli_power_off_cluster) =
      cli_power_off.subcommand_matches("cluster")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let hsm_group_name_arg = cli_power_off_cluster
        .get_one::<String>("CLUSTER_NAME")
        .expect("The 'cluster name' argument must have a value");
      let force = cli_power_off_cluster
        .get_one::<bool>("graceful")
        .expect("The 'graceful' argument must have a value");
      let output: &str =
        cli_power_off_cluster.get_one::<String>("output").unwrap();
      let target_hsm_group_vec = get_groups_names_available(
        backend,
        &shasta_token,
        Some(hsm_group_name_arg),
        settings_hsm_group_name_opt,
      )
      .await?;
      let target_hsm_group = target_hsm_group_vec
        .first()
        .expect("The 'cluster name' argument must have a value");
      let assume_yes: bool = cli_power_off_cluster.get_flag("assume-yes");
      power_off_cluster::exec(
        backend,
        &shasta_token,
        target_hsm_group,
        *force,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    } else if let Some(cli_power_off_node) =
      cli_power_off.subcommand_matches("nodes")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let xname_requested: &str = cli_power_off_node
        .get_one::<String>("VALUE")
        .expect("The 'xnames' argument must have values");
      let force = cli_power_off_node
        .get_one::<bool>("graceful")
        .expect("The 'graceful' argument must have a value");
      let assume_yes: bool = cli_power_off_node.get_flag("assume-yes");
      let output: &str =
        cli_power_off_node.get_one::<String>("output").unwrap();
      power_off_nodes::exec(
        backend,
        &shasta_token,
        xname_requested,
        *force,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    }
  } else if let Some(cli_power_reset) = cli_power.subcommand_matches("reset") {
    if let Some(cli_power_reset_cluster) =
      cli_power_reset.subcommand_matches("cluster")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let hsm_group_name_arg = cli_power_reset_cluster
        .get_one::<String>("CLUSTER_NAME")
        .expect("The 'cluster name' argument must have a value");
      let force = cli_power_reset_cluster
        .get_one::<bool>("graceful")
        .expect("The 'graceful' argument must have a value");
      let output: &str =
        cli_power_reset_cluster.get_one::<String>("output").unwrap();
      let target_hsm_group_vec = get_groups_names_available(
        backend,
        &shasta_token,
        Some(hsm_group_name_arg),
        settings_hsm_group_name_opt,
      )
      .await?;
      let target_hsm_group = target_hsm_group_vec
        .first()
        .expect("Power off cluster must operate against a cluster");
      let assume_yes: bool = cli_power_reset_cluster.get_flag("assume-yes");
      power_reset_cluster::exec(
        backend.clone(),
        &shasta_token,
        target_hsm_group,
        *force,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    } else if let Some(cli_power_reset_node) =
      cli_power_reset.subcommand_matches("nodes")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let xname_requested: &str = cli_power_reset_node
        .get_one::<String>("VALUE")
        .expect("The 'xnames' argument must have values");
      let force = cli_power_reset_node
        .get_one::<bool>("graceful")
        .expect("The 'graceful' argument must have a value");
      let assume_yes: bool = cli_power_reset_node.get_flag("assume-yes");
      let output: &str =
        cli_power_reset_node.get_one::<String>("output").unwrap();
      power_reset_nodes::exec(
        backend,
        &shasta_token,
        xname_requested,
        *force,
        assume_yes,
        output,
        kafka_audit_opt,
      )
      .await?;
    }
  }
  Ok(())
}
