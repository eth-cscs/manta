//! Implements the `manta get templates` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use crate::output;
use manta_shared::types::params::template::GetTemplateParams;

/// Parse CLI arguments into typed [`GetTemplateParams`].
fn parse_template_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetTemplateParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  GetTemplateParams {
    name: cli_args.opt_string("name"),
    group_name: cli_args.opt_string("group"),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    limit,
  }
}

/// CLI adapter for `manta get templates`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_template_params(cli_args, ctx.settings_hsm_group_name_opt);

  let templates = MantaClient::from_app_ctx(ctx)?
    .get_templates(token, &params)
    .await?;

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
