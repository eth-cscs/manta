//! IMS image backend methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::ims::{Image, PatchImage};

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// List IMS images, optionally restricted to a single id.
  pub async fn get_images(
    &self,
    token: &str,
    id: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    self.backend.get_images(token, id).await
  }

  /// Delete an IMS image by id.
  pub async fn delete_image(
    &self,
    token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    self.backend.delete_image(token, image_id).await
  }

  /// Patch IMS image metadata (link / arch / tags).
  pub async fn update_image(
    &self,
    token: &str,
    image_id: &str,
    patch: &PatchImage,
  ) -> Result<(), Error> {
    self.backend.update_image(token, image_id, patch).await
  }

  /// Filter images in place using the backend's per-site rules.
  pub fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    self.backend.filter_images(image_vec)
  }
}
