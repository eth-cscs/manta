
use manta_backend_dispatcher::{
  error::Error,
  interfaces::get_images_and_details::GetImagesAndDetailsTrait,
  types::ims::Image,
};

use StaticBackendDispatcher::*;


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
    match self {
      CSM(b) => {
        b.get_images_and_details(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          id_opt,
          limit_number,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_images_and_details(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_name_vec,
          id_opt,
          limit_number,
        )
        .await
      }
    }
  }
}
