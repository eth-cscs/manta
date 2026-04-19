use anyhow::{Error, bail};

use crate::common::{self, app_context::AppContext};
use crate::service;
use crate::service::template::ApplyTemplateParams;

/// Create a BOS session template and optionally boot.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  bos_session_name_opt: Option<&str>,
  bos_sessiontemplate_name: &str,
  bos_session_operation: &str,
  limit: &str,
  include_disabled: bool,
  assume_yes: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let params = ApplyTemplateParams {
    bos_session_name: bos_session_name_opt.map(str::to_string),
    bos_sessiontemplate_name: bos_sessiontemplate_name.to_string(),
    bos_session_operation: bos_session_operation.to_string(),
    limit: limit.to_string(),
    include_disabled,
  };

  let (bos_session, limit_vec) =
    service::template::validate_and_prepare_template_session(
      &ctx.infra,
      token,
      &params,
    )
    .await?;

  // Ask user for confirmation
  let operation = if bos_session_operation.to_lowercase() == "boot" {
    "reboot (if necessary)"
  } else {
    bos_session_operation
  };

  if !common::user_interaction::confirm(
    &format!(
      "{}\nThe nodes above will {}. \
       Please confirm to proceed?",
      limit_vec.join(","),
      operation
    ),
    assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  if dry_run {
    println!(
      "Dry-run enabled. No changes persisted \
       into the system"
    );
    println!("BOS session info:\n{:#?}", bos_session);
    Ok(())
  } else {
    let created = service::template::create_bos_session(
      &ctx.infra,
      token,
      bos_session,
    )
    .await?;

    println!(
      "BOS session '{}' for BOS \
       sessiontemplate '{}' created.\n\
       Please wait a few minutes for BOS \
       session to start.",
      created.name.unwrap_or_default(),
      bos_sessiontemplate_name
    );
    Ok(())
  }
}
