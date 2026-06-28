//! Implements the `manta get group-nodes` command.
//!
//! Hits `GET /groups/nodes` on `manta-server` to enumerate the nodes
//! belonging to an HSM group. Output mirrors `manta get nodes`: a
//! cluster status summary, one-line CSV of NIDs or xnames, JSON, or a
//! [`crate::output::node`] table. See [`super::nodes`] for the
//! host-expression variant.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::cluster::GetClusterParams;

/// Parse CLI arguments into typed [`GetClusterParams`].
///
/// The positional `HSM_GROUP_NAME` takes precedence over the default
/// group from `cli.toml`.
fn parse_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetClusterParams {
  GetClusterParams {
    group_name: cli_args.opt_string("HSM_GROUP_NAME"),
    settings_group_name: settings_hsm_group_name_opt.map(String::from),
    status_filter: cli_args.opt_string("status"),
  }
}

/// CLI adapter for `manta get group-nodes`.
///
/// Consumes clap matches for the `group-nodes` subcommand (positional
/// `HSM_GROUP_NAME`, `--status`, `--output`, `--nids-only-one-line`,
/// `--xnames-only-one-line`, `--summary-status`), calls the server
/// once, and renders the response in the requested form.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, JSON serialisation
/// fails, or `--output` holds an unrecognised value.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_cluster_params(cli_args, ctx.settings_group_name_opt);
  let nids_only = cli_args.get_flag("nids-only-one-line");
  let xnames_only = cli_args.get_flag("xnames-only-one-line");
  let output_opt = cli_args.opt_str("output");
  let summary_status = cli_args.get_flag("summary-status");

  let hsm = params
    .group_name
    .as_deref()
    .or(params.settings_group_name.as_deref());

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let node_details_list = client
    .openapi
    .get_groups_nodes(hsm, params.status_filter.as_deref(), client.site_name())
    .await
    .into_anyhow()?;

  output::node::render_node_details(
    node_details_list,
    nids_only,
    xnames_only,
    summary_status,
    output_opt,
  )
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
    assert_eq!(params.group_name.as_deref(), Some("compute"));
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
    assert_eq!(params.settings_group_name.as_deref(), Some("default-group"));
  }
}
