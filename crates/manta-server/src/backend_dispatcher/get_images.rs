//! `GetImagesAndDetailsTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl GetImagesAndDetailsTrait for StaticBackendDispatcher {
  async fn get_images_and_details(
    &self,
    token: &str,
    group_group_name_vec: &[String],
    id_opt: Option<&str>,
    limit_number: Option<&u8>,
  ) -> Result<Vec<(Image, String, String, bool)>, Error> {
    dispatch!(
      self,
      get_images_and_details,
      token,
      group_group_name_vec,
      id_opt,
      limit_number
    )
  }
}
