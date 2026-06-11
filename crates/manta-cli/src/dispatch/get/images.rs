//! Implements the `manta get images` command.

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::dto::Image;
use manta_shared::types::api::image::GetImagesParams;

/// Parse CLI arguments into typed [`GetImagesParams`].
fn parse_images_params(cli_args: &clap::ArgMatches) -> GetImagesParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  GetImagesParams {
    id: cli_args.opt_string("id"),
    pattern: cli_args.opt_string("pattern"),
    limit,
  }
}

/// CLI adapter for `manta get images`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_images_params(cli_args);

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let raw = client
    .openapi
    .get_images(
      params.id.as_deref(),
      params.limit.map(i32::from),
      params.pattern.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  // The server returns IMS images as a JSON array. Deserialize into
  // manta-shared's `Image` type so the renderer keeps working.
  let images: Vec<Image> = raw
    .into_iter()
    .map(serde_json::from_value)
    .collect::<Result<Vec<_>, _>>()
    .context("Failed to deserialize IMS images list")?;

  output::image::print(&images);

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn images_cmd() -> clap::Command {
    crate::build::get::subcommand_get_images()
  }

  #[test]
  fn parse_no_args() {
    let matches = images_cmd().get_matches_from(["images"]);
    let params = parse_images_params(&matches);
    assert!(params.id.is_none());
    assert!(params.pattern.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches = images_cmd().get_matches_from(["images", "--most-recent"]);
    let params = parse_images_params(&matches);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_id() {
    let matches = images_cmd().get_matches_from(["images", "--id", "abc-123"]);
    let params = parse_images_params(&matches);
    assert_eq!(params.id.as_deref(), Some("abc-123"));
  }
}
