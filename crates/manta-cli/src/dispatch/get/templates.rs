//! Implements the `manta get templates` command.
//!
//! Hits `GET /templates` on `manta-server` to list BOS session
//! templates. The server returns the BOS payload as `serde_json::Value`
//! since the upstream type lacks `ToSchema`; the handler deserialises
//! into [`BosSessionTemplate`] before rendering via
//! [`crate::output::template`].

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::template::GetTemplateParams;
use manta_shared::types::dto::BosSessionTemplate;

/// Parse CLI arguments into typed [`GetTemplateParams`].
///
/// `--most-recent` forces `limit = Some(1)`, overriding any explicit
/// `--limit` value. `settings_hsm_group_name_opt` is the default group
/// from `cli.toml`, used as a fallback when `--group` is omitted.
fn parse_template_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetTemplateParams {
  let limit = cli_args.limit_or_most_recent();

  GetTemplateParams {
    name: cli_args.opt_string("name"),
    group_name: cli_args.opt_string("group"),
    settings_group_name: settings_hsm_group_name_opt.map(String::from),
    limit,
  }
}

/// CLI adapter for `manta get templates`.
///
/// Consumes clap matches for the `templates` subcommand (`--name`,
/// `--group`, `--limit`, `--most-recent`, required `--output`), calls
/// the server once, and prints either the renderer's output or a "no
/// templates found" notice when the list is empty.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the `serde_json::Value`
/// payload cannot be deserialised into [`BosSessionTemplate`] values,
/// the required `--output` flag is missing, or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_template_params(cli_args, ctx.settings_group_name_opt);

  let group_name = params.effective_group();

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let raw = client
    .openapi
    .get_templates(
      group_name,
      params.limit.map(i32::from),
      params.name.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  // Server returns the BOS session template list as `serde_json::Value`;
  // deserialize into the manta-shared typed shape so the renderer can
  // use its accessor methods unchanged.
  let templates: Vec<BosSessionTemplate> = serde_json::from_value(raw)
    .context("Failed to deserialise BOS session template list")?;

  let output_opt = cli_args.req_str("output")?;

  if templates.is_empty() {
    println!("No BOS template found!");
  } else {
    output::template::print(&templates, output_opt)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn template_cmd() -> clap::Command {
    crate::build::get::subcommand_get_bos_template()
  }

  #[test]
  fn parse_no_args() {
    let matches = template_cmd().get_matches_from(["templates"]);
    let params = parse_template_params(&matches, None);
    assert!(params.name.is_none());
    assert!(params.group_name.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches =
      template_cmd().get_matches_from(["templates", "--most-recent"]);
    let params = parse_template_params(&matches, None);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_name_filter() {
    let matches =
      template_cmd().get_matches_from(["templates", "--name", "my-template"]);
    let params = parse_template_params(&matches, None);
    assert_eq!(params.name.as_deref(), Some("my-template"));
  }

  #[test]
  fn parse_limit() {
    let matches =
      template_cmd().get_matches_from(["templates", "--limit", "5"]);
    let params = parse_template_params(&matches, None);
    assert_eq!(params.limit, Some(5));
  }
}
