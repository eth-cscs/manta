use anyhow::Error;

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::image::{self, GetImagesParams};

/// Parse CLI arguments into typed [`GetImagesParams`].
fn parse_images_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetImagesParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  GetImagesParams {
    id: cli_args.get_one::<String>("id").cloned(),
    hsm_group: cli_args
      .try_get_one::<String>("hsm-group")
      .ok()
      .flatten()
      .cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    limit,
  }
}

/// CLI adapter for `manta get images`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_images_params(cli_args, ctx.settings_hsm_group_name_opt);

  let images = image::get_images(
    ctx.backend,
    token,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    &params,
  )
  .await?;

  output::image::print(&images);

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{arg, value_parser};

  fn images_cmd() -> clap::Command {
    clap::Command::new("images")
      .arg(arg!(--id <ID> "image id"))
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
      .arg(arg!(-m --"most-recent" "most recent"))
      .arg(
        arg!(-l --limit <VALUE> "limit")
          .value_parser(value_parser!(u8).range(1..)),
      )
  }

  #[test]
  fn parse_no_args() {
    let matches = images_cmd().get_matches_from(["images"]);
    let params = parse_images_params(&matches, None);
    assert!(params.id.is_none());
    assert!(params.hsm_group.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches = images_cmd().get_matches_from(["images", "--most-recent"]);
    let params = parse_images_params(&matches, None);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_id() {
    let matches = images_cmd().get_matches_from(["images", "--id", "abc-123"]);
    let params = parse_images_params(&matches, None);
    assert_eq!(params.id.as_deref(), Some("abc-123"));
  }
}
