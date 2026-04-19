use anyhow::{Context, Error, bail};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::hardware::{
  self, GetHardwareClusterParams,
  calculate_hsm_hw_component_summary, get_cluster_hw_pattern,
};

/// Parse CLI arguments into typed [`GetHardwareClusterParams`].
fn parse_hardware_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetHardwareClusterParams {
  GetHardwareClusterParams {
    hsm_group_name: cli_args.get_one::<String>("CLUSTER_NAME").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get hardware cluster`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_hardware_cluster_params(cli_args, ctx.cli.settings_hsm_group_name_opt);
  let output_opt = cli_args.get_one::<String>("output").map(String::as_str);

  let result =
    hardware::get_hardware_cluster(&ctx.infra, token, &params)
      .await?;

  if output_opt.is_some_and(|o| o.eq("json")) {
    for node_summary in &result.node_summaries {
      println!(
        "{}",
        serde_json::to_string_pretty(node_summary)
          .context("Failed to serialize node summary")?
      );
    }
  } else if output_opt.is_some_and(|o| o.eq("pattern")) {
    let pattern = get_cluster_hw_pattern(result.node_summaries);
    output::hardware::print_to_terminal_cluster_hw_pattern(
      &result.hsm_group_name,
      pattern,
    );
  } else if output_opt.is_some_and(|o| o.eq("details")) {
    output::hardware::print_table_details(&result.node_summaries);
  } else if output_opt.is_some_and(|o| o.eq("summary")) {
    let summary =
      calculate_hsm_hw_component_summary(&result.node_summaries);
    output::hardware::print_table_summary(&summary);
  } else {
    bail!("'output' value not valid");
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::arg;

  fn hw_cluster_cmd() -> clap::Command {
    clap::Command::new("cluster")
      .arg(arg!([CLUSTER_NAME] "cluster name"))
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .value_parser(["json", "pattern", "details", "summary"]),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = hw_cluster_cmd().get_matches_from(["cluster"]);
    let params = parse_hardware_cluster_params(&matches, None);
    assert!(params.hsm_group_name.is_none());
    assert!(params.settings_hsm_group_name.is_none());
  }

  #[test]
  fn parse_cluster_name() {
    let matches = hw_cluster_cmd().get_matches_from(["cluster", "compute"]);
    let params = parse_hardware_cluster_params(&matches, None);
    assert_eq!(params.hsm_group_name.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_settings_hsm_group() {
    let matches = hw_cluster_cmd().get_matches_from(["cluster"]);
    let params = parse_hardware_cluster_params(&matches, Some("default"));
    assert_eq!(params.settings_hsm_group_name.as_deref(), Some("default"));
  }
}
