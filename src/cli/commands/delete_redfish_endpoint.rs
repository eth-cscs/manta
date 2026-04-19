use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service::redfish_endpoints;

/// CLI adapter for `manta delete redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  redfish_endpoints::delete_redfish_endpoint(&ctx.infra, token, id).await?;
  println!("Redfish endpoint for id '{}' deleted successfully", id);
  Ok(())
}
