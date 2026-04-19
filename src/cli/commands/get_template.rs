use anyhow::{Context, Error};

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::template::{self, GetTemplateParams};

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
    name: cli_args.get_one::<String>("name").cloned(),
    hsm_group: cli_args
      .try_get_one::<String>("hsm-group")
      .ok()
      .flatten()
      .cloned(),
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

  let templates = template::get_templates(
    ctx.backend,
    token,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    &params,
  )
  .await?;

  let output_opt: &String = cli_args
    .get_one("output")
    .context("output must be a valid value")?;

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
  use clap::{arg, value_parser};

  fn template_cmd() -> clap::Command {
    clap::Command::new("templates")
      .arg(arg!(-n --name <NAME> "template name"))
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .default_value("table")
          .value_parser(["json", "table"]),
      )
      .arg(arg!(-m --"most-recent" "most recent"))
      .arg(
        arg!(-l --limit <VALUE> "limit")
          .value_parser(value_parser!(u8).range(1..)),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = template_cmd().get_matches_from(["templates"]);
    let params = parse_template_params(&matches, None);
    assert!(params.name.is_none());
    assert!(params.hsm_group.is_none());
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
