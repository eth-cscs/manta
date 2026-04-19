use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::common::audit;
use crate::service::boot_parameters::{self, UpdateBootParametersParams};

/// CLI adapter for `manta update boot-parameters`.
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

  boot_parameters::update_boot_parameters(&ctx.infra, token, params).await?;

  // Audit
  audit::maybe_send_audit(
    ctx.cli.kafka_audit_opt,
    token,
    "Update boot parameters",
    Some(serde_json::json!(hosts)),
    None,
  )
  .await;

  Ok(())
}
