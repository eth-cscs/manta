use manta_backend_dispatcher::{
  error::Error,
  interfaces::ims::{GetImagesAndDetailsTrait, ImsTrait},
  types::ims::{Image, PatchImage},
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ImsTrait for StaticBackendDispatcher {
  async fn get_images(
    &self,
    shasta_token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    dispatch!(self, get_images, shasta_token, image_id_opt)
  }

  async fn get_all_images(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
  ) -> Result<Vec<Image>, Error> {
    dispatch!(
      self,
      get_all_images,
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    )
  }

  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    dispatch!(sync self, filter_images, image_vec)
  }

  async fn update_image(
    &self,
    shasta_token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    dispatch!(self, update_image, shasta_token, image_id, image)
  }

  async fn delete_image(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_id: &str,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete_image,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      image_id
    )
  }
}

impl GetImagesAndDetailsTrait for StaticBackendDispatcher {
  async fn get_images_and_details(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &[String],
    id_opt: Option<&str>,
    limit_number: Option<&u8>,
  ) -> Result<Vec<(Image, String, String, bool)>, Error> {
    dispatch!(
      self,
      get_images_and_details,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
      id_opt,
      limit_number
    )
  }
}
