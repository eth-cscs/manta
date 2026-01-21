
use manta_backend_dispatcher::{
  error::Error,
  interfaces::ims::ImsTrait,
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
    match self {
      CSM(b) => b.get_images(shasta_token, image_id_opt).await,
      OCHAMI(b) => b.get_images(shasta_token, image_id_opt).await,
    }
  }

  async fn get_all_images(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
  ) -> Result<Vec<Image>, Error> {
    match self {
      CSM(b) => {
        b.get_all_images(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
      OCHAMI(b) => {
        b.get_all_images(shasta_token, shasta_base_url, shasta_root_cert)
          .await
      }
    }
  }

  fn filter_images(&self, image_vec: &mut Vec<Image>) -> Result<(), Error> {
    match self {
      CSM(b) => b.filter_images(image_vec),
      OCHAMI(b) => b.filter_images(image_vec),
    }
  }

  async fn update_image(
    &self,
    shasta_token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.update_image(shasta_token, image_id, image).await,
      OCHAMI(b) => b.update_image(shasta_token, image_id, image).await,
    }
  }

  async fn delete_image(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_id: &str,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.delete_image(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id,
        )
        .await
      }
      OCHAMI(b) => {
        b.delete_image(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id,
        )
        .await
      }
    }
  }
}
