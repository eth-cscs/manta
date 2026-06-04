//! Implements the `manta get group-nodes` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use crate::output;
use manta_shared::types::cluster_status;
use manta_shared::types::params::cluster::GetClusterParams;

/// Parse CLI arguments into typed [`GetClusterParams`].
fn parse_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetClusterParams {
  GetClusterParams {
    hsm_group_name: cli_args.opt_string("HSM_GROUP_NAME"),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    status_filter: cli_args.opt_string("status"),
  }
}

/// CLI adapter for `manta get group-nodes`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_cluster_params(cli_args, ctx.settings_hsm_group_name_opt);
  let nids_only = cli_args.get_flag("nids-only-one-line");
  let xnames_only = cli_args.get_flag("xnames-only-one-line");
  let output_opt = cli_args.opt_str("output");
  let summary_status = cli_args.get_flag("summary-status");

  let server_url = ctx.manta_server_url;
  let node_details_list = MantaClient::new(server_url, ctx.site_name)?
    .get_group_nodes(token, &params)
    .await?;

  if summary_status {
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
  } else if xnames_only {
    let node_xname_list: Vec<String> = node_details_list
      .iter()
      .map(|nd| nd.xname.clone())
      .collect();

    if output_opt == Some("json") {
      println!(
        "{}",
        serde_json::to_string(&node_xname_list)
          .context("Failed to serialize node xname list")?
      );
    } else {
      println!("{}", node_xname_list.join(","));
    }
  } else {
    match output_opt {
      Some("json") => {
        println!(
          "{}",
          serde_json::to_string_pretty(&node_details_list)
            .context("Failed to serialize node details")?
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

  fn cluster_cmd() -> clap::Command {
    crate::build::get::subcommand_get_group_nodes()
  }

  #[test]
  fn parse_positional_only_leaves_status_filter_unset() {
    let matches = cluster_cmd().get_matches_from(["group-nodes", "compute"]);
    let params = parse_cluster_params(&matches, None);
    assert_eq!(params.hsm_group_name.as_deref(), Some("compute"));
    assert!(params.status_filter.is_none());
  }

  #[test]
  fn parse_status_filter() {
    let matches = cluster_cmd().get_matches_from([
      "group-nodes",
      "compute",
      "--status",
      "ON",
    ]);
    let params = parse_cluster_params(&matches, None);
    assert_eq!(params.status_filter.as_deref(), Some("ON"));
  }

  #[test]
  fn parse_settings_hsm_group_preserved_alongside_positional() {
    let matches = cluster_cmd().get_matches_from(["group-nodes", "compute"]);
    let params = parse_cluster_params(&matches, Some("default-group"));
    assert_eq!(
      params.settings_hsm_group_name.as_deref(),
      Some("default-group")
    );
  }
}
