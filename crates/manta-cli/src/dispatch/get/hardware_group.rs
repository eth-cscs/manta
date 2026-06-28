//! Implements the `manta get hardware group` command.
//!
//! Hits `GET /groups/hardware` on `manta-server` to return the
//! aggregated hardware inventory of an HSM group. Output defaults to
//! the `summary` view from [`crate::output::hardware::print_cluster`];
//! pass `--output` for an alternative view. See
//! [`super::hardware_nodes`] for the per-node variant.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::hardware::GetHardwareClusterParams;

/// Parse CLI arguments into typed [`GetHardwareClusterParams`].
///
/// The positional `GROUP_NAME` takes precedence over the default group
/// from `cli.toml`.
fn parse_hardware_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetHardwareClusterParams {
  GetHardwareClusterParams {
    group_name: cli_args.get_one::<String>("GROUP_NAME").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get hardware group`.
///
/// Consumes clap matches for the `hardware group` subcommand
/// (positional `GROUP_NAME`, optional `--output`), fetches the
/// aggregated inventory, and hands the JSON to
/// [`crate::output::hardware::print_cluster`].
///
/// # Errors
///
/// Returns an error if the HTTP request fails or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_hardware_cluster_params(cli_args, ctx.settings_group_name_opt);
  let output = cli_args
    .get_one::<String>("output")
    .map_or("summary", String::as_str);

  let hsm = params.effective_group();

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let json = client
    .openapi
    .get_groups_hardware(hsm, client.site_name())
    .await
    .into_anyhow()?;

  output::hardware::print_cluster(&json, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hw_group_cmd() -> clap::Command {
    crate::build::get::subcommand_get_hardware_group()
  }

  #[test]
  fn parse_positional_only_leaves_settings_unset() {
    let matches = hw_group_cmd().get_matches_from(["group", "compute"]);
    let params = parse_hardware_cluster_params(&matches, None);
    assert_eq!(params.group_name.as_deref(), Some("compute"));
    assert!(params.settings_hsm_group_name.is_none());
  }

  #[test]
  fn parse_settings_hsm_group_preserved_alongside_positional() {
    let matches = hw_group_cmd().get_matches_from(["group", "compute"]);
    let params = parse_hardware_cluster_params(&matches, Some("default"));
    assert_eq!(params.settings_hsm_group_name.as_deref(), Some("default"));
  }
}
