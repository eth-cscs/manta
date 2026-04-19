use anyhow::{Context, Error};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::hardware::{self, GetHardwareNodeParams};

/// Parse CLI arguments into typed [`GetHardwareNodeParams`].
fn parse_hardware_node_params(cli_args: &clap::ArgMatches) -> Result<GetHardwareNodeParams, Error> {
  let xnames = cli_args
    .get_one::<String>("XNAMES")
    .context("The 'XNAMES' argument must have a value")?
    .clone();

  Ok(GetHardwareNodeParams {
    xnames,
    type_artifact: cli_args.get_one::<String>("type").cloned(),
  })
}

/// CLI adapter for `manta get hardware node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_hardware_node_params(cli_args)?;
  let output_opt = cli_args.get_one::<String>("output").map(String::as_str);

  let result =
    hardware::get_hardware_node(ctx.backend, token, &params).await?;

  if output_opt.is_some_and(|o| o.eq("json")) {
    println!(
      "{}",
      serde_json::to_string_pretty(&result.node_summary)
        .context("Failed to serialize node summary")?
    );
  } else {
    output::hardware::print_node_table(&[result.node_summary]);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::arg;

  fn hw_node_cmd() -> clap::Command {
    clap::Command::new("node")
      .arg(arg!(<XNAMES> "xnames"))
      .arg(arg!(-t --type <TYPE> "artifact type"))
      .arg(arg!(-o --output <FORMAT> "output format"))
  }

  #[test]
  fn parse_xnames_only() {
    let matches = hw_node_cmd().get_matches_from(["node", "x1000c0s0b0n0"]);
    let params = parse_hardware_node_params(&matches).unwrap();
    assert_eq!(params.xnames, "x1000c0s0b0n0");
    assert!(params.type_artifact.is_none());
  }

  #[test]
  fn parse_with_type() {
    let matches = hw_node_cmd()
      .get_matches_from(["node", "x1000c0s0b0n0", "--type", "Processors"]);
    let params = parse_hardware_node_params(&matches).unwrap();
    assert_eq!(params.type_artifact.as_deref(), Some("Processors"));
  }
}
