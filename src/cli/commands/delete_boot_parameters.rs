use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service::boot_parameters;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  boot_parameters::delete_boot_parameters(&ctx.infra, token, hosts).await?;
  println!("Boot parameters deleted successfully");
  Ok(())
}
