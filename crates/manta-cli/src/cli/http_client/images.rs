//! IMS image endpoints: list, bulk-delete.

use anyhow::Context;
use serde_json::Value;

use manta_shared::shared::dto::Image;
use manta_shared::shared::params::image::GetImagesParams;

use super::{MantaClient, QueryBuilder};

/// Image entry as returned by `GET /api/v1/images`.
#[derive(serde::Deserialize)]
struct ImageEntry {
  image: serde_json::Value,
  configuration_name: String,
  image_id: String,
  is_linked: bool,
}

impl MantaClient {
  pub async fn get_images(
    &self,
    token: &str,
    params: &GetImagesParams,
  ) -> anyhow::Result<Vec<(Image, String, String, bool)>> {
    let q = QueryBuilder::new()
      .opt("id", &params.id)
      .opt("hsm_group", &params.hsm_group)
      .opt_display("limit", &params.limit)
      .build();
    let entries: Vec<ImageEntry> = self.get_json(token, "/images", &q).await?;
    entries
      .into_iter()
      .map(|e| {
        let img: Image = serde_json::from_value(e.image)
          .context("Failed to deserialize image")?;
        Ok((img, e.configuration_name, e.image_id, e.is_linked))
      })
      .collect()
  }

  pub async fn delete_images(
    &self,
    token: &str,
    ids: &[&str],
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let q = [("ids", ids.join(",")), ("dry_run", dry_run.to_string())];
    self.delete_json_with_query(token, "/images", &q).await
  }
}
