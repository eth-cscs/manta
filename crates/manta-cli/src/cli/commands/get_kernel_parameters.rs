//! Implements the `manta get kernel-parameters` command.

use anyhow::{Context, Error, bail};

use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::http_client::MantaClient;
use crate::cli::output;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::kernel_parameters::GetKernelParametersParams;

/// Parse CLI arguments into typed [`GetKernelParametersParams`].
fn parse_kernel_parameters_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetKernelParametersParams {
  GetKernelParametersParams {
    hsm_group: cli_args.opt_string("group"),
    nodes: cli_args.opt_string("nodes"),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get kernel-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_kernel_parameters_params(cli_args, ctx.settings_hsm_group_name_opt);

  let server_url = ctx.manta_server_url;
  let boot_parameters = MantaClient::new(server_url, ctx.site_name)?
    .get_kernel_parameters(token, &params)
    .await?;

  let output = cli_args.req_str("output")?;
  let filter_opt = cli_args.opt_str("filter");

  match output {
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&boot_parameters)
        .context("Failed to serialize boot parameters to JSON")?
    ),
    "table" => {
      output::kernel_parameters::print_table(boot_parameters, filter_opt)
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
  use clap::arg;

  fn kernel_params_cmd() -> clap::Command {
    clap::Command::new("kernel-parameters")
      .arg(arg!(-H --group <HSM_GROUP_NAME> "hsm group"))
      .arg(arg!(-n --nodes <NODES> "nodes"))
      .arg(arg!(-f --filter <FILTER> "filter"))
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .default_value("table")
          .value_parser(["json", "table"]),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = kernel_params_cmd().get_matches_from(["kernel-parameters"]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert!(params.hsm_group.is_none());
    assert!(params.nodes.is_none());
  }

  #[test]
  fn parse_hsm_group() {
    let matches = kernel_params_cmd().get_matches_from([
      "kernel-parameters",
      "--group",
      "compute",
    ]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert_eq!(params.hsm_group.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_nodes() {
    let matches = kernel_params_cmd().get_matches_from([
      "kernel-parameters",
      "--nodes",
      "x1000c0s0b0n0",
    ]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert_eq!(params.nodes.as_deref(), Some("x1000c0s0b0n0"));
  }
}
