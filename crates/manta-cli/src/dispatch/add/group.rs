//! Implements the `manta add group` command.
//!
//! `POST /api/v1/groups` creates the group. If `--nodes` is provided
//! the leaf follows up with `POST /api/v1/groups/{label}/members` to
//! seed initial membership. Both calls are issued only after the
//! operator confirms the JSON preview shown on stdout
//! (`--assume-yes` skips the prompt). `--dry-run` skips both POSTs and
//! prints the preview through
//! [`crate::output::action_result::print_with_data`] so `-o json`
//! still works.

use anyhow::{Context, Error, bail};

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{AddNodesToGroupRequest, Group};
use crate::output::action_result;

/// Arguments for [`exec`] — mirrors the clap flags on
/// `manta add group`.
pub struct ExecParams<'a> {
  /// Group label (required).
  pub label: &'a str,
  /// Free-text description.
  pub description: Option<&'a str>,
  /// Optional `--nodes` host expression to seed initial members via a
  /// follow-up `POST /api/v1/groups/{label}/members`.
  pub hosts_expression: Option<&'a str>,
  /// When `true`, skip the interactive confirmation prompt.
  pub assume_yes: bool,
  /// When `true`, print the preview only and skip every server call.
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// CLI adapter for `manta add group`.
///
/// # Errors
///
/// Returns an error when the group payload fails to serialise for the
/// preview, when the user declines the confirmation prompt, when the
/// HTTP client cannot be built, when the `POST /groups` call fails, or
/// when the optional `POST /groups/{label}/members` follow-up fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  auth_token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let grp = Group {
    label: p.label.to_string(),
    description: p.description.map(String::from),
    tags: None,
    members: None,
    exclusive_group: Some("false".to_string()),
  };

  let add_nodes_req = p.hosts_expression.map(|expr| AddNodesToGroupRequest {
    hosts_expression: expr.to_string(),
  });

  if p.dry_run {
    // Two payloads in one preview — route through `print_with_data` so
    // the dry-run honours `-o json` like every other mutating verb.
    let preview = match &add_nodes_req {
      Some(req) => serde_json::json!({
        "create_group": &grp,
        "add_nodes_to_group": req,
      }),
      None => serde_json::json!({ "create_group": &grp }),
    };
    return action_result::print_with_data(
      "Would create group (and add nodes if provided):",
      &preview,
      p.output,
    );
  }

  if !common::confirm::confirm(
    &format!(
      "This operation will create the group below:\n{}\nPlease confirm to proceed",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    ),
    p.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  let client = MantaClient::from_app_ctx(ctx, Some(auth_token))?;
  client
    .openapi
    .create_group(client.site_name(), &grp)
    .await
    .into_anyhow()?;

  if let Some(req) = add_nodes_req {
    client
      .openapi
      .add_nodes_to_group(p.label, client.site_name(), &req)
      .await
      .into_anyhow()?;
  }

  action_result::print(&format!("Group '{}' created", p.label), p.output)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta add group` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "group",
      "--label",
      "compute",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `add group`: {result:?}"
    );
  }

  /// `-d` short alias parses (newly available after the swap of `-d/--description` to `-D`).
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta", "add", "group", "--label", "compute", "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse on `add group`: {result:?}"
    );
  }

  /// `-D` (capital) is the description short alias after the swap.
  /// Regression guard: a future change must NOT collapse this back to `-d`.
  #[test]
  fn description_short_alias_is_capital_d() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "group",
      "--label",
      "compute",
      "-D",
      "ops cluster",
    ]);
    assert!(
      result.is_ok(),
      "expected -D to parse as --description on `add group`: {result:?}"
    );
  }
}
