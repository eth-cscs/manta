use crate::common::{
  app_context::AppContext, audit, authentication::get_api_token,
  authorization::validate_target_hsm_members,
};
use anyhow::{Context, Error};
use manta_backend_dispatcher::{
  interfaces::bss::BootParametersTrait, types::bss::BootParameters,
};

/// Update boot parameters for specified nodes.
pub async fn exec(
  ctx: &AppContext<'_>,
  xnames: &str,
  nids: Option<&str>,
  macs: Option<&str>,
  params: Option<&str>,
  kernel: Option<&str>,
  initrd: Option<&str>,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let kafka_audit_opt = ctx.kafka_audit_opt;

  println!("Update boot parameters");

  let shasta_token = get_api_token(backend, site_name).await?;

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
  let params: String = params.unwrap_or_default().to_string();
  let kernel: String = kernel.unwrap_or_default().to_string();
  let initrd: String = initrd.unwrap_or_default().to_string();

  validate_target_hsm_members(backend, &shasta_token, &hosts).await?;

  let boot_parameters = BootParameters {
    hosts: hosts.clone(),
    macs,
    nids,
    params,
    kernel,
    initrd,
    cloud_init: None,
  };

  log::debug!("new boot params: {:#?}", boot_parameters);

  backend
    .update_bootparameters(&shasta_token, &boot_parameters)
    .await?;

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    audit::send_audit(
      kafka_audit,
      &shasta_token,
      "Update boot parameters",
      Some(serde_json::json!(hosts)),
      None,
    )
    .await;
  }

  Ok(())
}
