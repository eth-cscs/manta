use crate::common::{self, app_context::AppContext};
use crate::service;

/// Delete or cancel a CFS session.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  session_name: &str,
  dry_run: bool,
  assume_yes: bool,
) -> Result<(), anyhow::Error> {
  let deletion_ctx = service::session::prepare_session_deletion(
    &ctx.infra,
    token,
    session_name,
    ctx.cli.settings_hsm_group_name_opt,
  )
  .await?;

  if !common::user_interaction::confirm(
    &format!(
      "Session '{}' will get canceled:\nDo you want to continue?",
      session_name,
    ),
    assume_yes,
  ) {
    println!("Operation cancelled by user");
    return Ok(());
  }

  if !deletion_ctx.image_ids.is_empty()
    && !common::user_interaction::confirm(
      &format!(
        "Images listed below which will get deleted:\n{}\nDo you want to continue?",
        deletion_ctx.image_ids.join("\n"),
      ),
      assume_yes,
    )
  {
    println!("Operation cancelled by user");
    return Ok(());
  }

  service::session::execute_session_deletion(
    &ctx.infra,
    token,
    &deletion_ctx,
    dry_run,
  )
  .await?;

  Ok(())
}
