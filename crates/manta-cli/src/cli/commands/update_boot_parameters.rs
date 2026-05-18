//! Implements the `manta update boot-parameters` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;
use manta_shared::common::audit;
use manta_shared::shared::params::boot_parameters::UpdateBootParametersParams;

/// CLI adapter for `manta update boot-parameters`.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  xnames: &str,
  nids: Option<&str>,
  macs: Option<&str>,
  boot_params: Option<&str>,
  kernel: Option<&str>,
  initrd: Option<&str>,
) -> Result<(), Error> {
  println!("Update boot parameters");

  let hosts: Vec<String> = xnames.split(',').map(String::from).collect();
  let macs: Option<Vec<String>> =
    macs.map(|x| x.split(',').map(String::from).collect());
  let nids: Option<Vec<u32>> = nids
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
    params: boot_params.unwrap_or_default().to_string(),
    kernel: kernel.unwrap_or_default().to_string(),
    initrd: initrd.unwrap_or_default().to_string(),
  };

  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .update_boot_parameters(token, &params)
    .await?;

  // Audit
  audit::maybe_send_audit(
    ctx.kafka_audit_opt,
    token,
    "Update boot parameters",
    Some(serde_json::json!(hosts)),
    None,
  )
  .await;

  Ok(())
}
