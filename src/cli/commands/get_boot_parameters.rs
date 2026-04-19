use anyhow::Error;

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::boot_parameters::{self, GetBootParametersParams};

/// Parse CLI arguments into typed [`GetBootParametersParams`].
fn parse_boot_parameters_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetBootParametersParams {
  GetBootParametersParams {
    hsm_group: cli_args.get_one::<String>("hsm-group").cloned(),
    nodes: cli_args.get_one::<String>("nodes").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get boot-parameters`.
///
/// Parses CLI arguments into typed parameters, delegates to
/// the service layer, and formats the output.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_boot_parameters_params(cli_args, ctx.settings_hsm_group_name_opt);

  let boot_parameters =
    boot_parameters::get_boot_parameters(ctx.backend, token, &params).await?;

  output::boot_parameters::print(&boot_parameters, None)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::arg;

  fn boot_params_cmd() -> clap::Command {
    clap::Command::new("boot-parameters")
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
      .arg(arg!(-n --nodes <NODES> "nodes"))
  }

  #[test]
  fn parse_no_args() {
    let matches = boot_params_cmd().get_matches_from(["boot-parameters"]);
    let params = parse_boot_parameters_params(&matches, None);
    assert!(params.hsm_group.is_none());
    assert!(params.nodes.is_none());
    assert!(params.settings_hsm_group_name.is_none());
  }

  #[test]
  fn parse_hsm_group() {
    let matches = boot_params_cmd()
      .get_matches_from(["boot-parameters", "--hsm-group", "compute"]);
    let params = parse_boot_parameters_params(&matches, None);
    assert_eq!(params.hsm_group.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_nodes() {
    let matches = boot_params_cmd()
      .get_matches_from(["boot-parameters", "--nodes", "x1000c0s0b0n0"]);
    let params = parse_boot_parameters_params(&matches, None);
    assert_eq!(params.nodes.as_deref(), Some("x1000c0s0b0n0"));
  }

  #[test]
  fn parse_settings_hsm_group() {
    let matches = boot_params_cmd().get_matches_from(["boot-parameters"]);
    let params = parse_boot_parameters_params(&matches, Some("default-group"));
    assert_eq!(params.settings_hsm_group_name.as_deref(), Some("default-group"));
  }
}
