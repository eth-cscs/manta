//! Routes `manta apply *` subcommands to their exec functions.

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::{dispatch, http_client::MantaClient};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

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
        dispatch::apply::hardware_group::exec(m, ctx, &token).await?
      }
      Some((other, _)) => bail!("Unknown 'apply hardware' subcommand: {other}"),
      None => bail!("No 'apply hardware' subcommand provided"),
    },

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

      dispatch::apply::sat_file::exec::exec(
        ctx,
        &token,
        &dispatch::apply::sat_file::exec::SatApplyOptions {
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
      let _ = assume_yes;
      dispatch::apply::template::exec(
        ctx,
        &token,
        dispatch::apply::template::ExecParams {
          session_name: bos_session_name_opt,
          template_name: bos_sessiontemplate_name,
          operation: bos_session_operation,
          limit,
          include_disabled,
          dry_run: m.get_flag("dry-run"),
          output: m.opt_str("output"),
        },
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

    Some(("boot-parameters", m)) => {
      let hosts = m.req_str("hosts")?;
      let params = m.opt_str("params");
      let kernel = m.opt_str("kernel");
      let initrd = m.opt_str("initrd");
      let output_opt = m.opt_str("output");
      dispatch::apply::boot_parameters::exec(
        ctx,
        &token,
        dispatch::apply::boot_parameters::ExecParams {
          xnames: hosts,
          nids: None,
          macs: None,
          boot_params: params,
          kernel,
          initrd,
          output: output_opt,
        },
      )
      .await?;
    }

    Some(("redfish-endpoints", m)) => {
      let id = m
        .opt_string("id")
        .context("The 'id' argument is mandatory")?;
      let params = manta_shared::types::params::redfish_endpoints::UpdateRedfishEndpointParams {
        id,
        name: m.opt_string("name"),
        hostname: m.opt_string("hostname"),
        domain: m.opt_string("domain"),
        fqdn: m.opt_string("fqdn"),
        enabled: m.get_flag("enabled"),
        user: m.opt_string("user"),
        password: m.opt_string("password"),
        use_ssdp: m.get_flag("use-ssdp"),
        mac_required: m.get_flag("mac-required"),
        mac_addr: m.opt_string("macaddr"),
        ip_address: m.opt_string("ipaddress"),
        rediscover_on_update: m.get_flag("rediscover-on-update"),
        template_id: m.opt_string("template-id"),
      };
      dispatch::apply::redfish_endpoint::exec(
        ctx,
        &token,
        params,
        m.opt_str("output"),
      )
      .await?;
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
      let _ = (assume_yes, do_not_reboot);
      dispatch::apply::kernel_parameters::exec(
        ctx,
        &token,
        dispatch::apply::kernel_parameters::ExecParams {
          kernel_params: kernel_parameters,
          hosts_expression: nodes_opt,
          hsm_group: hsm_group_name_arg_opt,
          dry_run: dryrun,
          output: output_opt,
        },
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
        let _ = (m.get_flag("assume-yes"), m.get_flag("do-not-reboot"));
        dispatch::apply::boot_node::exec(
          ctx,
          &token,
          dispatch::apply::boot_node::ExecParams {
            boot_image: new_boot_image_id_opt,
            boot_image_configuration: m.opt_str("boot-image-configuration"),
            runtime_configuration: m.opt_str("runtime-configuration"),
            kernel_parameters: m.opt_str("kernel-parameters"),
            hosts_expression: hosts_string,
            dry_run: m.get_flag("dry-run"),
            output: m.opt_str("output"),
          },
        )
        .await?;
      }
      Some(("group", m)) => {
        let _ = (m.get_flag("assume-yes"), m.get_flag("do-not-reboot"));
        dispatch::apply::boot_group::exec(
          ctx,
          &token,
          dispatch::apply::boot_group::ExecParams {
            boot_image: m.opt_str("boot-image"),
            boot_image_configuration: m.opt_str("boot-image-configuration"),
            runtime_configuration: m.opt_str("runtime-configuration"),
            kernel_parameters: m.opt_str("kernel-parameters"),
            hsm_group_name: m.req_str("CLUSTER_NAME")?,
            dry_run: m.get_flag("dry-run"),
            output: m.opt_str("output"),
          },
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
