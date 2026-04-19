use crate::cli::commands::{
  delete_and_cancel_session, delete_boot_parameters,
  delete_configurations_and_derivatives, delete_group,
  delete_hw_component_cluster, delete_images, delete_kernel_parameters,
  delete_node, delete_redfish_endpoint,
};
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta delete` subcommands (group, node,
/// session, configuration, image, boot-parameters,
/// kernel-parameters, hardware, redfish-endpoint).
pub async fn handle_delete(
  cli_delete: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  if let Some(cli_delete_group) = cli_delete.subcommand_matches("group") {
    let label: &String = cli_delete_group
      .get_one("VALUE")
      .context("Group name argument is mandatory")?;
    let force: bool = *cli_delete_group
      .get_one("force")
      .context("'force' argument must have a value")?;
    delete_group::exec(
      ctx,
      &token,
      label,
      force,
      ctx.cli.kafka_audit_opt,
    )
    .await?;
  } else if let Some(cli_delete_node) = cli_delete.subcommand_matches("node") {
    let id: &String = cli_delete_node
      .get_one("VALUE")
      .context("Group name argument is mandatory")?;
    delete_node::exec(ctx, &token, id).await?;
  } else if let Some(cli_delete_hw_configuration) =
    cli_delete.subcommand_matches("hardware")
  {
    let dryrun = cli_delete_hw_configuration.get_flag("dry-run");
    let delete_hsm_group =
      cli_delete_hw_configuration.get_flag("delete-hsm-group");
    let target_hsm_group_name_arg_opt = cli_delete_hw_configuration
      .get_one::<String>("target-cluster")
      .map(String::as_str);
    let parent_hsm_group_name_arg_opt = cli_delete_hw_configuration
      .get_one::<String>("parent-cluster")
      .map(String::as_str);
    let pattern = cli_delete_hw_configuration
      .get_one::<String>("pattern")
      .context("'pattern' argument is mandatory")?;

    delete_hw_component_cluster::exec(
      ctx,
      target_hsm_group_name_arg_opt,
      parent_hsm_group_name_arg_opt,
      pattern,
      dryrun,
      delete_hsm_group,
    )
    .await?;
  } else if let Some(cli_delete_boot_parameters) =
    cli_delete.subcommand_matches("boot-parameters")
  {
    let xnames = cli_delete_boot_parameters.get_one::<String>("hosts");
    let hosts: Vec<String> = xnames
      .map(String::as_str)
      .unwrap_or_default()
      .split(',')
      .map(String::from)
      .collect();
    delete_boot_parameters::exec(ctx, &token, hosts).await?;
  } else if let Some(cli_delete_redfish_endpoint) =
    cli_delete.subcommand_matches("redfish-endpoint")
  {
    let id: &String = cli_delete_redfish_endpoint.get_one("id").context(
      "Host argument is mandatory. \
         Please provide the host to delete",
    )?;
    delete_redfish_endpoint::exec(ctx, &token, id).await?;
  } else if let Some(cli_delete_kernel_parameters) =
    cli_delete.subcommand_matches("kernel-parameters")
  {
    let hsm_group_name_arg_opt = cli_delete_kernel_parameters
      .get_one::<String>("hsm-group")
      .map(String::as_str);
    let nodes = cli_delete_kernel_parameters
      .get_one::<String>("nodes")
      .map(String::as_str);
    let kernel_parameters = cli_delete_kernel_parameters
      .get_one::<String>("VALUE")
      .context("'VALUE' argument is mandatory")?;
    let assume_yes: bool = cli_delete_kernel_parameters.get_flag("assume-yes");
    let do_not_reboot: bool =
      cli_delete_kernel_parameters.get_flag("do-not-reboot");
    let dryrun = cli_delete_kernel_parameters.get_flag("dry-run");

    delete_kernel_parameters::exec(
      ctx,
      hsm_group_name_arg_opt,
      nodes,
      kernel_parameters,
      assume_yes,
      do_not_reboot,
      dryrun,
    )
    .await?;
  } else if let Some(cli_delete_session) =
    cli_delete.subcommand_matches("session")
  {
    let session_name = cli_delete_session
      .get_one::<String>("SESSION_NAME")
      .context(
        "'session-name' argument \
         must be provided",
      )?;
    let assume_yes: bool = cli_delete_session.get_flag("assume-yes");
    let dry_run: bool = cli_delete_session.get_flag("dry-run");

    let result =
      delete_and_cancel_session::exec(ctx, session_name, dry_run, assume_yes)
        .await;

    if let Err(e) = result {
      bail!("Failed to delete session: {e}");
    }
  } else if let Some(cli_delete_configurations) =
    cli_delete.subcommand_matches("configurations")
  {
    let since_opt = if let Some(since) =
      cli_delete_configurations.get_one::<String>("since")
    {
      let date_time = chrono::NaiveDateTime::parse_from_str(
        &(since.to_string() + "T00:00:00"),
        "%Y-%m-%dT%H:%M:%S",
      )
      .context(format!(
        "Could not parse 'since' \
             date '{}'. Expected format: YYYY-MM-DD",
        since
      ))?;
      Some(date_time)
    } else {
      None
    };
    let until_opt = if let Some(until) =
      cli_delete_configurations.get_one::<String>("until")
    {
      let date_time = chrono::NaiveDateTime::parse_from_str(
        &(until.to_string() + "T00:00:00"),
        "%Y-%m-%dT%H:%M:%S",
      )
      .context(format!(
        "Could not parse 'until' \
             date '{}'. Expected format: YYYY-MM-DD",
        until
      ))?;
      Some(date_time)
    } else {
      None
    };
    let cfs_configuration_name_pattern: Option<&String> =
      cli_delete_configurations.get_one("configuration-name");
    let assume_yes = cli_delete_configurations.get_flag("assume-yes");

    let result = delete_configurations_and_derivatives::exec(
      ctx,
      cfs_configuration_name_pattern.map(String::as_str),
      since_opt,
      until_opt,
      assume_yes,
    )
    .await;
    if let Err(e) = result {
      bail!("Failed to delete configurations: {e}");
    }
  } else if let Some(cli_delete_images) =
    cli_delete.subcommand_matches("images")
  {
    let image_id_vec: Vec<&str> = cli_delete_images
      .get_one::<String>("IMAGE_LIST")
      .context(
        "'IMAGE_LIST' argument \
         must be provided",
      )?
      .split(',')
      .map(|image_id_str| image_id_str.trim())
      .collect();
    let dry_run: bool = cli_delete_images.get_flag("dry-run");

    match delete_images::command::exec(ctx, image_id_vec.as_slice(), dry_run)
      .await
    {
      Ok(_) => {
        println!("Images deleted successfully");
      }
      Err(e) => {
        bail!("Failed to delete images: {e}");
      }
    }
  } else {
    bail!("Unknown 'delete' subcommand");
  }
  Ok(())
}
