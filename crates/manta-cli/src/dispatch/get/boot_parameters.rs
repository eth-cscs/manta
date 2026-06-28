//! Implements the `manta get boot-parameters` command.
//!
//! Hits `GET /boot-parameters` on `manta-server` to read the BSS boot
//! parameters (kernel, initrd, params) registered for either a group or
//! a host expression. The configured default group from `cli.toml`
//! supplies the group name when neither `--group` nor `--nodes` is
//! given. Output is rendered by [`crate::output::boot_parameters`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::boot_parameters::GetBootParametersParams;

/// Parse CLI arguments into typed [`GetBootParametersParams`].
///
/// `settings_hsm_group_name_opt` is the default group from `cli.toml`,
/// used as a fallback when `--group` is omitted.
fn parse_boot_parameters_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetBootParametersParams {
  GetBootParametersParams {
    group_name: cli_args.opt_string("group"),
    host_expression: cli_args.opt_string("nodes"),
    settings_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get boot-parameters`.
///
/// Consumes clap matches for the `boot-parameters` subcommand
/// (`--group`, `--nodes`), issues a single
/// `MantaClient::openapi.get_boot_parameters` call, and hands the
/// response to [`crate::output::boot_parameters::print`].
///
/// # Errors
///
/// Returns an error if the HTTP client cannot be built, the server
/// rejects the request, or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_boot_parameters_params(cli_args, ctx.settings_group_name_opt);

  let group_name = params.effective_group();

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let boot_parameters = client
    .openapi
    .get_boot_parameters(
      group_name,
      params.host_expression.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  output::boot_parameters::print(&boot_parameters)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn boot_params_cmd() -> clap::Command {
    crate::build::get::subcommand_get_boot_parameters()
  }

  #[test]
  fn parse_nodes_only_leaves_group_unset() {
    let matches = boot_params_cmd().get_matches_from([
      "boot-parameters",
      "--nodes",
      "x1000c0s0b0n0",
    ]);
    let params = parse_boot_parameters_params(&matches, None);
    assert!(params.group_name.is_none());
    assert_eq!(params.host_expression.as_deref(), Some("x1000c0s0b0n0"));
    assert!(params.settings_group_name.is_none());
  }

  #[test]
  fn parse_hsm_group() {
    let matches = boot_params_cmd().get_matches_from([
      "boot-parameters",
      "--group",
      "compute",
    ]);
    let params = parse_boot_parameters_params(&matches, None);
    assert_eq!(params.group_name.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_settings_hsm_group_preserved_alongside_nodes() {
    let matches = boot_params_cmd().get_matches_from([
      "boot-parameters",
      "--nodes",
      "x1000c0s0b0n0",
    ]);
    let params = parse_boot_parameters_params(&matches, Some("default-group"));
    assert_eq!(params.settings_group_name.as_deref(), Some("default-group"));
  }
}
