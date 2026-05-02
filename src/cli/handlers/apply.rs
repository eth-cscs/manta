use crate::cli::commands;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta apply` subcommands (hardware, session,
/// sat-file, boot, template, ephemeral-env,
/// kernel-parameters).
pub async fn handle_apply(
  cli_apply: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  match cli_apply.subcommand() {
    Some(("hardware", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        commands::apply_hw_cluster::exec(m, ctx, &token).await?
      }
      Some((other, _)) => bail!("Unknown 'apply hardware' subcommand: {other}"),
      None => bail!("No 'apply hardware' subcommand provided"),
    },

    Some(("session", m)) => {
      let vault_base_url = ctx
        .infra
        .vault_base_url
        .context("vault_base_url is required for apply session")?;
      commands::apply_session::exec(m, ctx, &token, vault_base_url).await?
    }

    Some(("sat-file", m)) => {
      let vault_base_url = ctx
        .infra
        .vault_base_url
        .context("vault_base_url is required for apply sat-file")?;
      let k8s_api_url = ctx
        .infra
        .k8s_api_url
        .context("k8s_api_url is required for apply sat-file")?;

      let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

      let cli_value_vec_opt: Option<Vec<String>> =
        m.get_many("values").map(|value_vec| {
          value_vec
            .map(|value: &String| value.replace("__DATE__", &timestamp))
            .collect()
        });

      let cli_values_file_content_opt: Option<String> =
        if let Some(values_file_path) =
          m.get_one::<std::path::PathBuf>("values-file")
        {
          let content =
            std::fs::read_to_string(values_file_path).with_context(|| {
              format!(
                "Failed to read values file '{}'",
                values_file_path.display()
              )
            })?;
          Some(content.replace("__DATE__", &timestamp))
        } else {
          None
        };

      let sat_template_file = m
        .get_one::<std::path::PathBuf>("sat-template-file")
        .context("SAT template file argument not provided")?;

      let sat_file_content: String =
        std::fs::read_to_string(sat_template_file).with_context(|| {
          format!(
            "Could not read SAT file template '{}'",
            sat_template_file.display()
          )
        })?;

      let ansible_passthrough_env: Option<String> =
        ctx.cli.settings.get("ansible-passthrough").ok();
      let ansible_passthrough_cli_arg =
        m.get_one::<String>("ansible-passthrough").cloned();
      let ansible_passthrough =
        ansible_passthrough_env.or(ansible_passthrough_cli_arg);
      let ansible_verbosity: Option<u8> = m
        .get_one::<String>("ansible-verbosity")
        .map(|v| {
          v.parse::<u8>().with_context(|| {
            format!(
              "Could not parse ansible-verbosity '{}' as a number (0-255)",
              v
            )
          })
        })
        .transpose()?;

      let overwrite: bool = m.get_flag("overwrite-configuration");
      let prehook: Option<&String> = m.get_one("pre-hook");
      let posthook: Option<&String> = m.get_one("post-hook");
      let reboot: bool = m.get_flag("reboot");
      let watch_logs: bool = m.get_flag("watch-logs");
      let timestamps: bool = m.get_flag("timestamps");
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");

      let site = ctx
        .cli
        .configuration
        .sites
        .get(&ctx.cli.configuration.site)
        .ok_or_else(|| Error::msg("Site not valid"))?;

      let k8s_details = site
        .k8s
        .as_ref()
        .ok_or_else(|| Error::msg("K8s details not found in configuration"))?;

      commands::apply_sat_file::command::exec(
        ctx,
        &token,
        &commands::apply_sat_file::command::SatApplyOptions {
          vault_base_url,
          k8s_api_url,
          sat_file_content: sat_file_content.as_str(),
          values_file_content_opt: cli_values_file_content_opt.as_deref(),
          values_cli_opt: cli_value_vec_opt.as_deref(),
          ansible_verbosity_opt: ansible_verbosity,
          ansible_passthrough_opt: ansible_passthrough.as_deref(),
          reboot,
          watch_logs,
          timestamps,
          prehook_opt: prehook.map(String::as_str),
          posthook_opt: posthook.map(String::as_str),
          image_only: m.get_flag("image-only"),
          session_template_only: m.get_flag("sessiontemplate-only"),
          debug_on_failure: true,
          overwrite,
          dry_run,
          assume_yes,
          k8s: k8s_details,
        },
      )
      .await?;
    }

    Some(("template", m)) => {
      let bos_session_name_opt: Option<&String> = m.get_one("name");
      let bos_sessiontemplate_name: &String = m
        .get_one("template")
        .context("Template name is mandatory")?;
      let limit: &String = m.get_one("limit").context("Limit is mandatory")?;
      let bos_session_operation: &String = m
        .get_one("operation")
        .context("Operation is mandatory")?;
      let include_disabled: bool =
        *m.get_one("include-disabled").context(
          "'include-disabled' must have a value",
        )?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      commands::apply_template::exec(
        ctx,
        &token,
        bos_session_name_opt.map(String::as_str),
        bos_sessiontemplate_name,
        bos_session_operation,
        limit,
        include_disabled,
        assume_yes,
        dry_run,
      )
      .await?;
    }

    Some(("ephemeral-environment", m)) => {
      if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        bail!("This command needs to run in interactive mode");
      }
      commands::apply_ephemeral_env::exec(
        ctx.infra.shasta_base_url,
        ctx.infra.shasta_root_cert,
        &token,
        m.get_one::<String>("image-id")
          .context("'image-id' argument is mandatory")?,
      )
      .await?;
    }

    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt =
        m.get_one::<String>("hsm-group");
      let nodes_opt: Option<&String> = if hsm_group_name_arg_opt.is_none() {
        m.get_one::<String>("nodes")
      } else {
        None
      };
      let dryrun = m.get_flag("dry-run");
      let kernel_parameters = m
        .get_one::<String>("VALUE")
        .context("'VALUE' argument is mandatory")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let do_not_reboot: bool = m.get_flag("do-not-reboot");
      commands::apply_kernel_parameters::exec(
        ctx,
        &token,
        kernel_parameters,
        nodes_opt.map(|x| x.as_str()),
        hsm_group_name_arg_opt.map(|x| x.as_str()),
        assume_yes,
        do_not_reboot,
        dryrun,
      )
      .await?;
    }

    Some(("boot", m)) => match m.subcommand() {
      Some(("nodes", m)) => {
        let hosts_string: &str = m
          .get_one::<String>("VALUE")
          .context("'xnames' argument must have values")?;
        let new_boot_image_id_opt: Option<&String> = m.get_one("boot-image");
        if let Some(new_boot_image_id) = new_boot_image_id_opt
          && uuid::Uuid::parse_str(new_boot_image_id).is_err()
        {
          bail!("Image id is not a UUID");
        }
        let new_boot_image_configuration_opt: Option<&String> =
          m.get_one("boot-image-configuration");
        let new_runtime_configuration_opt: Option<&String> =
          m.get_one("runtime-configuration");
        let new_kernel_parameters_opt: Option<&String> =
          m.get_one::<String>("kernel-parameters");
        let assume_yes = m.get_flag("assume-yes");
        let do_not_reboot = m.get_flag("do-not-reboot");
        let dry_run = m.get_flag("dry-run");
        commands::apply_boot_node::exec(
          ctx,
          &token,
          new_boot_image_id_opt.map(String::as_str),
          new_boot_image_configuration_opt.map(String::as_str),
          new_runtime_configuration_opt.map(String::as_str),
          new_kernel_parameters_opt.map(String::as_str),
          hosts_string,
          assume_yes,
          do_not_reboot,
          dry_run,
        )
        .await?;
      }
      Some(("cluster", m)) => {
        let hsm_group_name_arg: &String = m
          .get_one("CLUSTER_NAME")
          .context("Cluster name must be provided")?;
        let new_boot_image_id_opt: Option<&String> = m.get_one("boot-image");
        let new_boot_image_configuration_opt: Option<&String> =
          m.get_one("boot-image-configuration");
        let new_runtime_configuration_opt: Option<&String> =
          m.get_one("runtime-configuration");
        let new_kernel_parameters_opt: Option<&String> =
          m.get_one("kernel-parameters");
        let assume_yes = m.get_flag("assume-yes");
        let do_not_reboot = m.get_flag("do-not-reboot");
        let dry_run = m.get_flag("dry-run");
        commands::apply_boot_cluster::exec(
          ctx,
          &token,
          new_boot_image_id_opt.map(String::as_str),
          new_boot_image_configuration_opt.map(String::as_str),
          new_runtime_configuration_opt.map(String::as_str),
          new_kernel_parameters_opt.map(String::as_str),
          hsm_group_name_arg,
          assume_yes,
          do_not_reboot,
          dry_run,
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'apply boot' subcommand: {other}"),
      None => bail!("No 'apply boot' subcommand provided"),
    },

    Some((other, _)) => bail!("Unknown 'apply' subcommand: {other}"),
    None => bail!("No 'apply' subcommand provided"),
  }
  Ok(())
}
