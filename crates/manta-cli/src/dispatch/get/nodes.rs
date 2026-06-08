//! Implements the `manta get nodes` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use crate::output;
use manta_shared::types::cluster_status;
use manta_shared::types::params::node::GetNodesParams;

/// Parse CLI arguments into typed [`GetNodesParams`].
fn parse_nodes_params(
  cli_args: &clap::ArgMatches,
) -> Result<GetNodesParams, Error> {
  let xname = cli_args.req_str("VALUE")?.to_string();

  Ok(GetNodesParams {
    host_expression: xname,
    include_siblings: cli_args.get_flag("include-siblings"),
    status_filter: cli_args.opt_string("status"),
  })
}

/// CLI adapter for `manta get nodes`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_nodes_params(cli_args)?;
  let nids_only = cli_args.get_flag("nids-only-one-line");
  let output_opt = cli_args.opt_str("output");
  let status_summary = cli_args.get_flag("summary-status");

  let node_details_list = MantaClient::from_app_ctx(ctx)?
    .get_nodes(token, &params)
    .await?;

  if status_summary {
    println!(
      "{}",
      cluster_status::compute_summary_status(&node_details_list)
    );
  } else if nids_only {
    let node_nid_list: Vec<String> =
      node_details_list.iter().map(|nd| nd.nid.clone()).collect();

    if output_opt == Some("json") {
      println!(
        "{}",
        serde_json::to_string(&node_nid_list)
          .context("Failed to serialize node NID list")?
      );
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else {
    match output_opt {
      Some("json") => {
        println!(
          "{}",
          serde_json::to_string_pretty(&node_details_list)
            .context("Failed to serialize node details list")?
        );
      }
      Some("summary") => {
        output::node::print_summary(node_details_list);
      }
      Some("table-wide") => {
        output::node::print_table(node_details_list, true);
      }
      Some("table") => {
        output::node::print_table(node_details_list, false);
      }
      _ => {
        bail!("Output value not recognized or missing");
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn nodes_cmd() -> clap::Command {
    crate::build::get::subcommand_get_node_details()
  }

  #[test]
  fn parse_xname_only() {
    let matches = nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0"]);
    let params = parse_nodes_params(&matches).unwrap();
    assert_eq!(params.host_expression, "x1000c0s0b0n0");
    assert!(!params.include_siblings);
    assert!(params.status_filter.is_none());
  }

  #[test]
  fn parse_with_siblings() {
    let matches = nodes_cmd().get_matches_from([
      "nodes",
      "x1000c0s0b0n0",
      "--include-siblings",
    ]);
    let params = parse_nodes_params(&matches).unwrap();
    assert!(params.include_siblings);
  }

  #[test]
  fn parse_with_status() {
    let matches = nodes_cmd().get_matches_from([
      "nodes",
      "x1000c0s0b0n0",
      "--status",
      "ON",
    ]);
    let params = parse_nodes_params(&matches).unwrap();
    assert_eq!(params.status_filter.as_deref(), Some("ON"));
  }
}
