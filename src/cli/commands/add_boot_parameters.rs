use anyhow::{Context, Error};
use manta_backend_dispatcher::types::bss::BootParameters;
use serde_json::Value;

use crate::common::app_context::AppContext;
use crate::service::boot_parameters;

/// CLI adapter for `manta add boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let hosts = cli_args
    .get_one::<String>("hosts")
    .context("'hosts' argument is mandatory")?;
  let macs: Option<String> = cli_args.get_one("macs").cloned();
  let nids: Option<String> = cli_args.get_one("nids").cloned();
  let params = cli_args
    .get_one::<String>("params")
    .context("'params' argument is mandatory")?
    .clone();
  let kernel = cli_args
    .get_one::<String>("kernel")
    .context("'kernel' argument is mandatory")?
    .clone();
  let initrd = cli_args
    .get_one::<String>("initrd")
    .context("'initrd' argument is mandatory")?
    .clone();
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
            format!(
              "Could not parse NID value '{}' as a number",
              value.trim()
            )
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

  boot_parameters::add_boot_parameters(&ctx.infra, token, &bp).await?;

  println!("Boot parameters created successfully");

  Ok(())
}
