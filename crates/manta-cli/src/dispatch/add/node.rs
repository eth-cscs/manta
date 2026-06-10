//! Implements the `manta add node` command.

use anyhow::Result;
use std::path::PathBuf;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::AddNodeRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub id: &'a str,
  pub group: &'a str,
  pub enabled: bool,
  pub arch: Option<String>,
  pub hardware_file: Option<&'a PathBuf>,
  pub output: Option<&'a str>,
}

/// CLI adapter for `manta add node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<()> {
  let _ = p.hardware_file;
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .add_node(
      client.site_name(),
      &AddNodeRequest {
        id: p.id.to_string(),
        group: p.group.to_string(),
        enabled: Some(p.enabled),
        arch: p.arch,
      },
    )
    .await
    .into_anyhow()?;

  action_result::print(
    &format!("Node '{}' created and added to group '{}'", p.id, p.group),
    p.output,
  )?;

  Ok(())
}
