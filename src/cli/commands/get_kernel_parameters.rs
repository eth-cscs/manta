use anyhow::{Context, Error, bail};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::kernel_parameters::{self, GetKernelParametersParams};

/// Parse CLI arguments into typed [`GetKernelParametersParams`].
fn parse_kernel_parameters_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetKernelParametersParams {
  GetKernelParametersParams {
    hsm_group: cli_args.get_one::<String>("hsm-group").cloned(),
    nodes: cli_args.get_one::<String>("nodes").cloned(),
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

  let boot_parameters =
    kernel_parameters::get_kernel_parameters(ctx.backend, token, &params)
      .await?;

  let output: &String = cli_args
    .get_one("output")
    .context("output value missing")?;
  let filter_opt = cli_args.get_one::<String>("filter").map(String::as_str);

  match output.as_str() {
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
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
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
    let matches = kernel_params_cmd()
      .get_matches_from(["kernel-parameters", "--hsm-group", "compute"]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert_eq!(params.hsm_group.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_nodes() {
    let matches = kernel_params_cmd()
      .get_matches_from(["kernel-parameters", "--nodes", "x1000c0s0b0n0"]);
    let params = parse_kernel_parameters_params(&matches, None);
    assert_eq!(params.nodes.as_deref(), Some("x1000c0s0b0n0"));
  }
}
