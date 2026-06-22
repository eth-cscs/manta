//! Implements the `manta get kernel-parameters` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::kernel_parameters::GetKernelParametersParams;

/// Parse CLI arguments into typed [`GetKernelParametersParams`].
fn parse_kernel_parameters_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetKernelParametersParams {
  GetKernelParametersParams {
    group_name: cli_args.opt_string("group"),
    nodes: cli_args.opt_string("nodes"),
    settings_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get kernel-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_kernel_parameters_params(cli_args, ctx.settings_group_name_opt);

  let group_name = params
    .group_name
    .as_deref()
    .or(params.settings_group_name.as_deref());

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let boot_parameters = client
    .openapi
    .get_kernel_parameters(
      group_name,
      params.nodes.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  let output = cli_args.req_str("output")?;
  let filter_opt = cli_args.opt_str("filter");

  match output {
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&boot_parameters)
        .context("Failed to serialize boot parameters to JSON")?
    ),
    "table" => {
      output::kernel_parameters::print_table(&boot_parameters, filter_opt);
    }
    _ => {
      bail!("'output' argument value missing or not supported");
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn kernel_params_cmd() -> clap::Command {
    crate::build::get::subcommand_get_kernel_parameters()
  }

  #[test]
  fn parse_nodes_only_leaves_group_unset() {
    let matches = kernel_params_cmd().get_matches_from([
      "kernel-parameters",
      "--nodes",
      "x1000c0s0b0n0",
    ]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert!(params.group_name.is_none());
    assert_eq!(params.nodes.as_deref(), Some("x1000c0s0b0n0"));
  }

  #[test]
  fn parse_hsm_group() {
    let matches = kernel_params_cmd().get_matches_from([
      "kernel-parameters",
      "--group",
      "compute",
    ]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert_eq!(params.group_name.as_deref(), Some("compute"));
  }
}
