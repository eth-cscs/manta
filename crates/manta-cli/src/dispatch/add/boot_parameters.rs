//! Implements the `manta add boot-parameters` command.
//!
//! Builds a [`BootParameters`] payload from `--hosts`, `--macs`,
//! `--nids`, `--params`, `--kernel`, `--initrd` (and optionally
//! `--cloud-init`) and POSTs it to `/api/v1/boot-parameters`. The
//! manta-shared `BootParameters` and the generated openapi type share
//! the JSON shape but differ in Rust-level optionality / integer width
//! (u32 vs i32); the bridge here is a serde round-trip.

use anyhow::{Context, Error};
use manta_shared::types::dto::BootParameters;
use serde_json::Value;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta add boot-parameters`.
///
/// # Errors
///
/// Returns an error when a required clap argument is missing, when any
/// NID fails to parse as `u32`, when the BootParameters fail to encode
/// for the wire, when the HTTP client cannot be built, or when the
/// server rejects the request.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let hosts = cli_args.req_str("hosts")?;
  let macs = cli_args.opt_string("macs");
  let nids = cli_args.opt_string("nids");
  let params = cli_args.req_str("params")?.to_string();
  let kernel = cli_args.req_str("kernel")?.to_string();
  let initrd = cli_args.req_str("initrd")?.to_string();
  let cloud_init = cli_args.get_one::<Value>("cloud-init").cloned();

  let host_vec: Vec<String> = hosts
    .split(',')
    .map(|value| value.trim().to_string())
    .collect();
  let mac_vec = macs.map(|x| {
    x.split(',')
      .map(|value| value.trim().to_string())
      .collect::<Vec<String>>()
  });
  let nid_vec: Option<Vec<u32>> = nids
    .map(|x| {
      x.split(',')
        .map(|value| {
          value.trim().parse().with_context(|| {
            format!("Could not parse NID value '{}' as a number", value.trim())
          })
        })
        .collect::<Result<Vec<u32>, _>>()
    })
    .transpose()?;

  let bp = BootParameters {
    hosts: host_vec,
    macs: mac_vec,
    nids: nid_vec,
    params,
    kernel,
    initrd,
    cloud_init,
  };

  // Convert from manta-shared's BootParameters to the generated
  // openapi_client::types::BootParameters via serde round-trip:
  // the two share the JSON wire shape but differ in Rust-level
  // field optionality / integer width (u32 vs i32).
  let bp_wire: crate::openapi_client::types::BootParameters =
    serde_json::from_value(
      serde_json::to_value(&bp)
        .context("Failed to serialize BootParameters")?,
    )
    .context("Failed to convert BootParameters to wire type")?;

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .add_boot_parameters(client.site_name(), &bp_wire)
    .await
    .into_anyhow()?;

  let output_opt = cli_args.opt_str("output");
  action_result::print("Boot parameters created successfully", output_opt)?;

  Ok(())
}
