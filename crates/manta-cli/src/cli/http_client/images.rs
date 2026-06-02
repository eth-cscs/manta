//! IMS image endpoints: list, bulk-delete.

use regex::Regex;
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
  ) -> anyhow::Result<Vec<Image>> {
    let q = QueryBuilder::new()
      .opt("id", &params.id)
      .opt("pattern", &params.pattern)
      .opt_display("limit", &params.limit)
      .build();

    let mut image_vec: Vec<Image> = self.get_json(token, "/images", &q).await?;

    if let Some(pattern) = &params.pattern {
      let re = Regex::new(&pattern)?;
      image_vec.retain(|image| re.is_match(&image.name));
    };

    if let Some(limit) = params.limit {
      image_vec.truncate(limit as usize);
    }

    Ok(image_vec)
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
