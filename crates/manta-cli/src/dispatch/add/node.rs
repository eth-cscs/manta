//! Implements the `manta add node` command.

use anyhow::Result;
use std::path::PathBuf;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
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
  MantaClient::from_app_ctx(ctx)?
    .add_node(token, p.id, p.group, p.enabled, p.arch)
    .await?;

  action_result::print(
    &format!("Node '{}' created and added to group '{}'", p.id, p.group),
    p.output,
  )?;

  Ok(())
}
