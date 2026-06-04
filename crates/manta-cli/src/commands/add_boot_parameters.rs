//! Implements the `manta add boot-parameters` command.

use anyhow::{Context, Error};
use manta_shared::shared::dto::BootParameters;
use serde_json::Value;

use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;

/// CLI adapter for `manta add boot-parameters`.
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

  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .add_boot_parameters(token, &bp)
    .await?;

  let output_opt = cli_args.opt_str("output");
  action_result::print("Boot parameters created successfully", output_opt)?;

  Ok(())
}
