//! Routes `manta apply *` subcommands to their exec functions.

use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::{commands, http_client::MantaClient};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch `manta apply` subcommands (hardware, session,
/// sat-file, boot, template, ephemeral-env,
/// kernel-parameters).
pub async fn handle_apply(
  cli_apply: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_apply.subcommand() {
    Some(("hardware", m)) => match m.subcommand() {
      Some(("group", m)) => {
        commands::apply_hw_group::exec(m, ctx, &token).await?
      }
      Some(("cluster", m)) => {
        eprintln!(
          "warning: 'manta apply hardware cluster' is deprecated; \
           use 'manta apply hardware group' instead.",
        );
        commands::apply_hw_group::exec(m, ctx, &token).await?
      }
      Some((other, _)) => bail!("Unknown 'apply hardware' subcommand: {other}"),
      None => bail!("No 'apply hardware' subcommand provided"),
    },

    Some(("session", m)) => {
      eprintln!(
        "warning: 'manta apply session' is deprecated; \
         use 'manta run session' instead.",
      );
      commands::apply_session::exec(m, ctx, &token).await?
    }

    Some(("sat-file", m)) => {
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
          let content = std::fs::read_to_string(values_file_path)
            .with_context(|| {
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

      let sat_file_content: String = std::fs::read_to_string(sat_template_file)
        .with_context(|| {
          format!(
            "Could not read SAT file template '{}'",
            sat_template_file.display()
          )
        })?;

      let ansible_passthrough_env: Option<String> =
        ctx.settings.get("ansible-passthrough").ok();
      let ansible_passthrough_cli_arg = m.opt_string("ansible-passthrough");
      let ansible_passthrough =
        ansible_passthrough_env.or(ansible_passthrough_cli_arg);
      let ansible_verbosity: Option<u8> = m
        .get_one::<String>("ansible-verbosity")
        .map(|v| {
          v.parse::<u8>().with_context(|| {
            format!(
              "Could not parse ansible-verbosity '{v}' as a number (0-255)"
            )
          })
        })
        .transpose()?;

      let overwrite: bool = m.get_flag("overwrite-configuration");
      let reboot: bool = m.get_flag("reboot");
      let watch_logs: bool = m.get_flag("watch-logs");
      let timestamps: bool = m.get_flag("timestamps");
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");

      commands::apply_sat_file::command::exec(
        ctx,
        &token,
        &commands::apply_sat_file::command::SatApplyOptions {
          sat_file_content: sat_file_content.as_str(),
          values_file_content_opt: cli_values_file_content_opt.as_deref(),
          values_cli_opt: cli_value_vec_opt.as_deref(),
          ansible_verbosity_opt: ansible_verbosity,
          ansible_passthrough_opt: ansible_passthrough.as_deref(),
          reboot,
          watch_logs,
          timestamps,
          prehook_opt: m.opt_str("pre-hook"),
          posthook_opt: m.opt_str("post-hook"),
          image_only: m.get_flag("image-only"),
          session_template_only: m.get_flag("sessiontemplate-only"),
          overwrite,
          dry_run,
          assume_yes,
          output_opt,
        },
      )
      .await?;
    }

    Some(("template", m)) => {
      let bos_session_name_opt = m.opt_str("name");
      let bos_sessiontemplate_name = m.req_str("template")?;
      let limit = m.req_str("limit")?;
      let bos_session_operation = m.req_str("operation")?;
      let include_disabled: bool = *m
        .get_one("include-disabled")
        .context("'include-disabled' must have a value")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      commands::apply_template::exec(
        ctx,
        &token,
        bos_session_name_opt,
        bos_sessiontemplate_name,
        bos_session_operation,
        limit,
        include_disabled,
        assume_yes,
        dry_run,
        output_opt,
      )
      .await?;
    }

    Some(("ephemeral-environment", m)) => {
      if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        bail!("This command needs to run in interactive mode");
      }
      let image_id = m.req_str("image-id")?;
      let server_url = ctx.manta_server_url;
      let response = MantaClient::new(server_url, ctx.site_name)?
        .create_ephemeral_env(&token, image_id)
        .await?;
      if let Some(hostname) = response.get("hostname").and_then(|v| v.as_str())
      {
        println!("{hostname}");
      }
    }

    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt = m.opt_str("group");
      let nodes_opt = if hsm_group_name_arg_opt.is_none() {
        m.opt_str("nodes")
      } else {
        None
      };
      let dryrun = m.get_flag("dry-run");
      let kernel_parameters = m.req_str("VALUE")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let do_not_reboot: bool = m.get_flag("do-not-reboot");
      let output_opt = m.opt_str("output");
      commands::apply_kernel_parameters::exec(
        ctx,
        &token,
        kernel_parameters,
        nodes_opt,
        hsm_group_name_arg_opt,
        assume_yes,
        do_not_reboot,
        dryrun,
        output_opt,
      )
      .await?;
    }

    Some(("boot", m)) => match m.subcommand() {
      Some(("nodes", m)) => {
        let hosts_string = m.req_str("VALUE")?;
        let new_boot_image_id_opt = m.opt_str("boot-image");
        if let Some(new_boot_image_id) = new_boot_image_id_opt
          && uuid::Uuid::parse_str(new_boot_image_id).is_err()
        {
          bail!("Image id is not a UUID");
        }
        let assume_yes = m.get_flag("assume-yes");
        let do_not_reboot = m.get_flag("do-not-reboot");
        let dry_run = m.get_flag("dry-run");
        let output_opt = m.opt_str("output");
        commands::apply_boot_node::exec(
          ctx,
          &token,
          new_boot_image_id_opt,
          m.opt_str("boot-image-configuration"),
          m.opt_str("runtime-configuration"),
          m.opt_str("kernel-parameters"),
          hosts_string,
          assume_yes,
          do_not_reboot,
          dry_run,
          output_opt,
        )
        .await?;
      }
      Some(("group", m)) => {
        let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
        let assume_yes = m.get_flag("assume-yes");
        let do_not_reboot = m.get_flag("do-not-reboot");
        let dry_run = m.get_flag("dry-run");
        let output_opt = m.opt_str("output");
        commands::apply_boot_group::exec(
          ctx,
          &token,
          m.opt_str("boot-image"),
          m.opt_str("boot-image-configuration"),
          m.opt_str("runtime-configuration"),
          m.opt_str("kernel-parameters"),
          hsm_group_name_arg,
          assume_yes,
          do_not_reboot,
          dry_run,
          output_opt,
        )
        .await?;
      }
      Some(("cluster", m)) => {
        eprintln!(
          "warning: 'manta apply boot cluster' is deprecated; \
           use 'manta apply boot group' instead.",
        );
        let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
        let assume_yes = m.get_flag("assume-yes");
        let do_not_reboot = m.get_flag("do-not-reboot");
        let dry_run = m.get_flag("dry-run");
        let output_opt = m.opt_str("output");
        commands::apply_boot_group::exec(
          ctx,
          &token,
          m.opt_str("boot-image"),
          m.opt_str("boot-image-configuration"),
          m.opt_str("runtime-configuration"),
          m.opt_str("kernel-parameters"),
          hsm_group_name_arg,
          assume_yes,
          do_not_reboot,
          dry_run,
          output_opt,
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
