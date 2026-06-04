//! Implements the `manta update boot-parameters` command.

use anyhow::{Context, Error};

use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;
use manta_shared::shared::params::boot_parameters::UpdateBootParametersParams;

pub struct ExecParams<'a> {
  pub xnames: &'a str,
  pub nids: Option<&'a str>,
  pub macs: Option<&'a str>,
  pub boot_params: Option<&'a str>,
  pub kernel: Option<&'a str>,
  pub initrd: Option<&'a str>,
  pub output: Option<&'a str>,
}

/// CLI adapter for `manta update boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let hosts: Vec<String> = p.xnames.split(',').map(String::from).collect();
  let macs: Option<Vec<String>> =
    p.macs.map(|x| x.split(',').map(String::from).collect());
  let nids: Option<Vec<u32>> = p
    .nids
    .map(|x| {
      x.split(',')
        .map(|nid| {
          nid.parse::<u32>().with_context(|| {
            format!("Invalid NID '{nid}': expected a positive integer")
          })
        })
        .collect::<Result<Vec<u32>, _>>()
    })
    .transpose()?;

  let params = UpdateBootParametersParams {
    hosts: hosts.clone(),
    nids,
    macs,
    params: p.boot_params.unwrap_or_default().to_string(),
    kernel: p.kernel.unwrap_or_default().to_string(),
    initrd: p.initrd.unwrap_or_default().to_string(),
  };

  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .update_boot_parameters(token, &params)
    .await?;

  action_result::print("Boot parameters updated", p.output)?;

  Ok(())
}
