use anyhow::{Context, Error, bail};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::node::{self, GetNodesParams};

/// Parse CLI arguments into typed [`GetNodesParams`].
fn parse_nodes_params(cli_args: &clap::ArgMatches) -> Result<GetNodesParams, Error> {
  let xname = cli_args
    .get_one::<String>("VALUE")
    .context("The 'xnames' argument must have values")?
    .clone();

  Ok(GetNodesParams {
    xname,
    include_siblings: cli_args.get_flag("include-siblings"),
    status_filter: cli_args.get_one::<String>("status").cloned(),
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
  let output_opt: Option<&String> = cli_args.get_one("output");
  let status_summary = cli_args.get_flag("summary-status");

  let node_details_list = node::get_nodes(
    ctx.backend,
    token,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    &params,
  )
  .await?;

  if status_summary {
    println!("{}", node::compute_summary_status(&node_details_list));
  } else if nids_only {
    let node_nid_list: Vec<String> = node_details_list
      .iter()
      .map(|nd| nd.nid.clone())
      .collect();

    if output_opt.is_some_and(|v| v == "json") {
      println!(
        "{}",
        serde_json::to_string(&node_nid_list)
          .context("Failed to serialize node NID list")?
      );
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else {
    match output_opt.map(String::as_str) {
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
  use clap::arg;

  fn nodes_cmd() -> clap::Command {
    clap::Command::new("nodes")
      .arg(arg!(<VALUE> "xname"))
      .arg(arg!(-s --status <STATUS> "status filter"))
      .arg(arg!(--"include-siblings" "include siblings"))
      .arg(arg!(--"nids-only-one-line" "nids only"))
      .arg(arg!(--"summary-status" "summary status"))
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .value_parser(["json", "table", "table-wide", "summary"]),
      )
  }

  #[test]
  fn parse_xname_only() {
    let matches = nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0"]);
    let params = parse_nodes_params(&matches).unwrap();
    assert_eq!(params.xname, "x1000c0s0b0n0");
    assert!(!params.include_siblings);
    assert!(params.status_filter.is_none());
  }

  #[test]
  fn parse_with_siblings() {
    let matches = nodes_cmd()
      .get_matches_from(["nodes", "x1000c0s0b0n0", "--include-siblings"]);
    let params = parse_nodes_params(&matches).unwrap();
    assert!(params.include_siblings);
  }

  #[test]
  fn parse_with_status() {
    let matches = nodes_cmd()
      .get_matches_from(["nodes", "x1000c0s0b0n0", "--status", "ON"]);
    let params = parse_nodes_params(&matches).unwrap();
    assert_eq!(params.status_filter.as_deref(), Some("ON"));
  }
}
