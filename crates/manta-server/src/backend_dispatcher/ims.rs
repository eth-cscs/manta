//! `ImsTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl ImsTrait for StaticBackendDispatcher {
  async fn get_images(
    &self,
    token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_images, token, image_id_opt)
  }

  async fn get_all_images(&self, token: &str) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_all_images, token)
  }

  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    dispatch!(sync self, filter_images, image_vec)
  }

  async fn update_image(
    &self,
    token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    dispatch!(self, update_image, token, image_id, image)
  }

  async fn delete_image(
    &self,
    token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_image, token, image_id)
  }
}
