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

  match cli_delete.subcommand() {
    Some(("group", m)) => {
      let label: &String = m
        .get_one("VALUE")
        .context("Group name argument is mandatory")?;
      let force: bool = *m
        .get_one("force")
        .context("'force' argument must have a value")?;
      delete_group::exec(ctx, &token, label, force, ctx.cli.kafka_audit_opt)
        .await?;
    }
    Some(("node", m)) => {
      let id: &String = m
        .get_one("VALUE")
        .context("Node id argument is mandatory")?;
      delete_node::exec(ctx, &token, id).await?;
    }
    Some(("hardware", m)) => {
      let dryrun = m.get_flag("dry-run");
      let delete_hsm_group = m.get_flag("delete-hsm-group");
      let target_hsm_group_name_arg_opt =
        m.get_one::<String>("target-cluster").map(String::as_str);
      let parent_hsm_group_name_arg_opt =
        m.get_one::<String>("parent-cluster").map(String::as_str);
      let pattern = m
        .get_one::<String>("pattern")
        .context("'pattern' argument is mandatory")?;
      delete_hw_component_cluster::exec(
        ctx,
        &token,
        target_hsm_group_name_arg_opt,
        parent_hsm_group_name_arg_opt,
        pattern,
        dryrun,
        delete_hsm_group,
      )
      .await?;
    }
    Some(("boot-parameters", m)) => {
      let xnames = m.get_one::<String>("hosts");
      let hosts: Vec<String> = xnames
        .map(String::as_str)
        .unwrap_or_default()
        .split(',')
        .map(String::from)
        .collect();
      delete_boot_parameters::exec(ctx, &token, hosts).await?;
    }
    Some(("redfish-endpoint", m)) => {
      let id: &String = m.get_one("id").context(
        "Host argument is mandatory. \
           Please provide the host to delete",
      )?;
      delete_redfish_endpoint::exec(ctx, &token, id).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt =
        m.get_one::<String>("hsm-group").map(String::as_str);
      let nodes = m.get_one::<String>("nodes").map(String::as_str);
      let kernel_parameters = m
        .get_one::<String>("VALUE")
        .context("'VALUE' argument is mandatory")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let do_not_reboot: bool = m.get_flag("do-not-reboot");
      let dryrun = m.get_flag("dry-run");
      delete_kernel_parameters::exec(
        ctx,
        &token,
        hsm_group_name_arg_opt,
        nodes,
        kernel_parameters,
        assume_yes,
        do_not_reboot,
        dryrun,
      )
      .await?;
    }
    Some(("session", m)) => {
      let session_name = m
        .get_one::<String>("SESSION_NAME")
        .context("'session-name' argument must be provided")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      if let Err(e) =
        delete_and_cancel_session::exec(ctx, &token, session_name, dry_run, assume_yes)
          .await
      {
        bail!("Failed to delete session: {e}");
      }
    }
    Some(("configurations", m)) => {
      let since_opt = if let Some(since) = m.get_one::<String>("since") {
        let date_time = chrono::NaiveDateTime::parse_from_str(
          &(since.to_string() + "T00:00:00"),
          "%Y-%m-%dT%H:%M:%S",
        )
        .context(format!(
          "Could not parse 'since' date '{}'. Expected format: YYYY-MM-DD",
          since
        ))?;
        Some(date_time)
      } else {
        None
      };
      let until_opt = if let Some(until) = m.get_one::<String>("until") {
        let date_time = chrono::NaiveDateTime::parse_from_str(
          &(until.to_string() + "T00:00:00"),
          "%Y-%m-%dT%H:%M:%S",
        )
        .context(format!(
          "Could not parse 'until' date '{}'. Expected format: YYYY-MM-DD",
          until
        ))?;
        Some(date_time)
      } else {
        None
      };
      let cfs_configuration_name_pattern: Option<&String> =
        m.get_one("configuration-name");
      let assume_yes = m.get_flag("assume-yes");
      if let Err(e) = delete_configurations_and_derivatives::exec(
        ctx,
        &token,
        cfs_configuration_name_pattern.map(String::as_str),
        since_opt,
        until_opt,
        assume_yes,
      )
      .await
      {
        bail!("Failed to delete configurations: {e}");
      }
    }
    Some(("images", m)) => {
      let image_id_vec: Vec<&str> = m
        .get_one::<String>("IMAGE_LIST")
        .context("'IMAGE_LIST' argument must be provided")?
        .split(',')
        .map(|s| s.trim())
        .collect();
      let dry_run: bool = m.get_flag("dry-run");
      match delete_images::command::exec(ctx, &token, image_id_vec.as_slice(), dry_run)
        .await
      {
        Ok(_) => println!("Images deleted successfully"),
        Err(e) => bail!("Failed to delete images: {e}"),
      }
    }
    Some((other, _)) => bail!("Unknown 'delete' subcommand: {other}"),
    None => bail!("No 'delete' subcommand provided"),
  }
  Ok(())
}
