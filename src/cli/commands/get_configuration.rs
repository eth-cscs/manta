use anyhow::{Context, Error, bail};

use crate::cli::output::configuration::{print_table_details_struct, print_table_struct};
use crate::common::app_context::AppContext;
use crate::service::configuration::{self, GetConfigurationParams};

/// Parse CLI arguments into typed [`GetConfigurationParams`].
fn parse_configuration_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetConfigurationParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  GetConfigurationParams {
    name: cli_args.get_one::<String>("name").cloned(),
    pattern: cli_args.get_one::<String>("pattern").cloned(),
    hsm_group: cli_args
      .try_get_one::<String>("hsm-group")
      .ok()
      .flatten()
      .cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    since: None,
    until: None,
    limit,
  }
}

/// CLI adapter for `manta get configurations`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_configuration_params(cli_args, ctx.settings_hsm_group_name_opt);

  let cfs_configuration_vec = configuration::get_configurations(
    ctx.backend,
    token,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    &params,
  )
  .await?;

  if cfs_configuration_vec.is_empty() {
    bail!("No CFS configuration found!");
  }

  let output_opt = cli_args.get_one::<String>("output").map(String::as_str);

  if output_opt.is_some_and(|o| o.eq("json")) {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_configuration_vec)
        .context("Failed to serialize CFS configurations to JSON")?
    );
  } else if cfs_configuration_vec.len() == 1 {
    let config = cfs_configuration_vec
      .first()
      .context("CFS configuration list unexpectedly empty")?;

    let vault_base_url = ctx
      .vault_base_url
      .context("vault base url is mandatory")?;

    let (details, cfs_session_vec_opt, bos_sessiontemplate_vec_opt, image_vec_opt) =
      configuration::get_configuration_details(
        ctx.backend,
        token,
        ctx.shasta_base_url,
        ctx.shasta_root_cert,
        ctx.gitea_base_url,
        vault_base_url,
        ctx.site_name,
        config,
      )
      .await?;

    print_table_details_struct(
      details,
      cfs_session_vec_opt,
      bos_sessiontemplate_vec_opt,
      image_vec_opt,
    );
  } else {
    print_table_struct(&cfs_configuration_vec);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{arg, value_parser};

  fn config_cmd() -> clap::Command {
    clap::Command::new("configurations")
      .arg(arg!(-n --name <NAME> "config name"))
      .arg(arg!(-p --pattern <PATTERN> "name pattern"))
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
      .arg(arg!(-m --"most-recent" "most recent"))
      .arg(
        arg!(-l --limit <VALUE> "limit")
          .value_parser(value_parser!(u8).range(1..)),
      )
      .arg(
        arg!(-o --output <FORMAT> "output format")
          .value_parser(["json", "table"]),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = config_cmd().get_matches_from(["configurations"]);
    let params = parse_configuration_params(&matches, None);
    assert!(params.name.is_none());
    assert!(params.pattern.is_none());
    assert!(params.hsm_group.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_name() {
    let matches =
      config_cmd().get_matches_from(["configurations", "--name", "my-config"]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.name.as_deref(), Some("my-config"));
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches =
      config_cmd().get_matches_from(["configurations", "--most-recent"]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_pattern() {
    let matches = config_cmd()
      .get_matches_from(["configurations", "--pattern", "compute-*"]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.pattern.as_deref(), Some("compute-*"));
  }
}
