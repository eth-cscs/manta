use anyhow::{Context, Error, bail};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::cluster::{self, GetClusterParams};
use crate::service::node;

/// Parse CLI arguments into typed [`GetClusterParams`].
fn parse_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetClusterParams {
  GetClusterParams {
    hsm_group_name: cli_args.get_one::<String>("HSM_GROUP_NAME").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    status_filter: cli_args.get_one::<String>("status").cloned(),
  }
}

/// CLI adapter for `manta get cluster`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_cluster_params(cli_args, ctx.cli.settings_hsm_group_name_opt);
  let nids_only = cli_args.get_flag("nids-only-one-line");
  let xnames_only = cli_args.get_flag("xnames-only-one-line");
  let output_opt: Option<&String> = cli_args.get_one("output");
  let summary_status = cli_args.get_flag("summary-status");

  let node_details_list = cluster::get_cluster_nodes(
    &ctx.infra,
    token,
    &params,
  )
  .await?;

  if summary_status {
    println!("{}", node::compute_summary_status(&node_details_list));
  } else if nids_only {
    let node_nid_list: Vec<String> = node_details_list
      .iter()
      .map(|nd| nd.nid.clone())
      .collect();

    if let Some(output) = output_opt
      && output.eq("json")
    {
      println!(
        "{}",
        serde_json::to_string(&node_nid_list)
          .context("Failed to serialize node NID list")?
      );
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else if xnames_only {
    let node_xname_list: Vec<String> = node_details_list
      .iter()
      .map(|nd| nd.xname.clone())
      .collect();

    if let Some(output) = output_opt
      && output.eq("json")
    {
      println!(
        "{}",
        serde_json::to_string(&node_xname_list)
          .context("Failed to serialize node xname list")?
      );
    } else {
      println!("{}", node_xname_list.join(","));
    }
  } else if let Some(output) = output_opt
    && output.eq("json")
  {
    println!(
      "{}",
      serde_json::to_string_pretty(&node_details_list)
        .context("Failed to serialize node details")?
    );
  } else if let Some(output) = output_opt
    && output.eq("summary")
  {
    output::node::print_summary(node_details_list);
  } else if let Some(output) = output_opt
    && output.eq("table-wide")
  {
    output::node::print_table(node_details_list, true);
  } else if let Some(output) = output_opt
    && output.eq("table")
  {
    output::node::print_table(node_details_list, false);
  } else {
    bail!("Output value not recognized or missing");
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::arg;

  fn cluster_cmd() -> clap::Command {
    clap::Command::new("cluster")
      .arg(arg!([HSM_GROUP_NAME] "hsm group name"))
      .arg(arg!(-s --status <STATUS> "status filter"))
      .arg(arg!(--"nids-only-one-line" "nids only"))
      .arg(arg!(--"xnames-only-one-line" "xnames only"))
      .arg(arg!(--"summary-status" "summary status"))
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .value_parser(["json", "table", "table-wide", "summary"]),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = cluster_cmd().get_matches_from(["cluster"]);
    let params = parse_cluster_params(&matches, None);
    assert!(params.hsm_group_name.is_none());
    assert!(params.status_filter.is_none());
  }

  #[test]
  fn parse_hsm_group() {
    let matches = cluster_cmd().get_matches_from(["cluster", "compute"]);
    let params = parse_cluster_params(&matches, None);
    assert_eq!(params.hsm_group_name.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_status_filter() {
    let matches =
      cluster_cmd().get_matches_from(["cluster", "--status", "ON"]);
    let params = parse_cluster_params(&matches, None);
    assert_eq!(params.status_filter.as_deref(), Some("ON"));
  }

  #[test]
  fn parse_settings_hsm_group() {
    let matches = cluster_cmd().get_matches_from(["cluster"]);
    let params = parse_cluster_params(&matches, Some("default-group"));
    assert_eq!(
      params.settings_hsm_group_name.as_deref(),
      Some("default-group")
    );
  }
}
