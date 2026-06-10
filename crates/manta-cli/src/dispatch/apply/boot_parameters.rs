//! Implements the `manta apply boot-parameters` command.

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::UpdateBootParametersParams;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub xnames: &'a str,
  pub nids: Option<&'a str>,
  pub macs: Option<&'a str>,
  pub boot_params: Option<&'a str>,
  pub kernel: Option<&'a str>,
  pub initrd: Option<&'a str>,
  pub output: Option<&'a str>,
}

/// CLI adapter for `manta apply boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let hosts: Vec<String> = p.xnames.split(',').map(String::from).collect();
  let macs: Option<Vec<String>> =
    p.macs.map(|x| x.split(',').map(String::from).collect());
  // Generated NIDs are i32; parsing as i32 matches the wire shape.
  let nids: Option<Vec<i32>> = p
    .nids
    .map(|x| {
      x.split(',')
        .map(|nid| {
          nid.parse::<i32>().with_context(|| {
            format!("Invalid NID '{nid}': expected an integer")
          })
        })
        .collect::<Result<Vec<i32>, _>>()
    })
    .transpose()?;

  let params = UpdateBootParametersParams {
    hosts,
    nids,
    macs,
    params: p.boot_params.unwrap_or_default().to_string(),
    kernel: p.kernel.unwrap_or_default().to_string(),
    initrd: p.initrd.unwrap_or_default().to_string(),
  };

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .update_boot_parameters(client.site_name(), &params)
    .await
    .into_anyhow()?;

  action_result::print("Boot parameters updated", p.output)?;

  Ok(())
}
