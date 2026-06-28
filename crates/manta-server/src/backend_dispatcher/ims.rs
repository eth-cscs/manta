//! [`ImsTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the CSM IMS (Image Management Service)
//! `/apis/ims/v3/images` API. Ochami's `impl ImsTrait for Ochami {}`
//! is empty, so each method on the Ochami branch returns
//! [`Error::Message`] from the trait default ("not implemented for
//! this backend").

use super::*;

impl ImsTrait for StaticBackendDispatcher {
  /// `GET /images/{id}` when `image_id_opt` is `Some`, otherwise
  /// the full image list. Returned as `Vec<Image>` for shape
  /// consistency with the list path.
  async fn get_images(
    &self,
    token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_images, token, image_id_opt)
  }

  /// `GET /images` — every image visible to the bearer.
  async fn get_all_images(&self, token: &str) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_all_images, token)
  }

  /// In-place client-side filter applied to `image_vec` (the only
  /// sync method on this trait). The backend strips images the
  /// caller cannot see or that fail a per-backend visibility rule.
  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    dispatch!(sync self, filter_images, image_vec)
  }

  /// `PATCH /images/{id}` — apply `image` as a partial update.
  async fn update_image(
    &self,
    token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    dispatch!(self, update_image, token, image_id, image)
  }

  /// `DELETE /images/{id}`.
  async fn delete_image(
    &self,
    token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_image, token, image_id)
  }
}
