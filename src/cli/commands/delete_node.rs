use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service::node;

/// CLI adapter for `manta delete node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  node::delete_node(&ctx.infra, token, id).await?;
  println!("Node deleted '{}'", id);
  Ok(())
}
