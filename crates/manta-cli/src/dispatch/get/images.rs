//! Implements the `manta get images` command.
//!
//! Fans out two parallel calls — `GET /images` and `GET /images/analysis`
//! — and joins them by IMS image id, then renders the list with a
//! `safe_to_delete` column via [`crate::output::image`]. Images are
//! keyed by IMS id (names are not unique because rebuilds reuse them).
//! Optional `--only-safe-to-delete` / `--only-unsafe-to-delete` filters
//! drop rows whose verdict does not match the requested kind; rows with
//! an unknown verdict (no IMS id, or no analysis row) are excluded.
//! Same orchestration pattern as [`super::configurations`].

use std::collections::HashMap;

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::image::GetImagesParams;
use manta_shared::types::dto::Image;

/// Parse CLI arguments into typed [`GetImagesParams`].
///
/// `--most-recent` forces `limit = Some(1)`, overriding any explicit
/// `--limit` value.
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
///
/// Consumes clap matches for the `images` subcommand (`--id`,
/// `--pattern`, `--limit`, `--most-recent`, `--only-safe-to-delete`,
/// `--only-unsafe-to-delete`), fetches the listing and analysis in
/// parallel, applies the optional safety filter, and renders the
/// merged table.
///
/// # Errors
///
/// Returns an error if either HTTP request fails or deserialising the
/// IMS images list into typed [`Image`] values fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_images_params(cli_args);

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let site = client.site_name();

  // Fan out: listing and deletion-safety analysis are independent.
  // The analysis endpoint already sequences its own upstream calls so
  // we don't compound load. Same pattern as `manta get configurations`.
  let (raw, safety_rows) = tokio::try_join!(
    async {
      client
        .openapi
        .get_images(
          params.id.as_deref(),
          params.limit.map(i32::from),
          params.pattern.as_deref(),
          site,
        )
        .await
        .into_anyhow()
    },
    async { client.openapi.get_image_analysis(site).await.into_anyhow() },
  )?;

  // The server returns IMS images as a JSON array. Deserialize into
  // manta-shared's `Image` type so the renderer keeps working.
  let images: Vec<Image> = raw
    .into_iter()
    .map(serde_json::from_value)
    .collect::<Result<Vec<_>, _>>()
    .context("Failed to deserialize IMS images list")?;

  // image_id -> safe_to_delete lookup. Images use their IMS id as the
  // unique key (names aren't unique — repeated builds keep the name).
  let safety: HashMap<String, bool> = safety_rows
    .into_iter()
    .map(|r| (r.image_id, r.safe_to_delete))
    .collect();

  // Optional safety filter. Images whose verdict is unknown (no row
  // in the analysis response, or no IMS id on the image) are excluded
  // from both filtered views; a filter only keeps rows whose verdict
  // matches the requested kind.
  let only_safe = cli_args.get_flag("only-safe-to-delete");
  let only_unsafe = cli_args.get_flag("only-unsafe-to-delete");
  let images: Vec<Image> = if only_safe || only_unsafe {
    images
      .into_iter()
      .filter(|img| {
        let safe = img.id.as_deref().and_then(|id| safety.get(id)).copied();
        (!only_safe || safe == Some(true))
          && (!only_unsafe || safe == Some(false))
      })
      .collect()
  } else {
    images
  };

  output::image::print(&images, &safety);

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
