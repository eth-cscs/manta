//! [`GetImagesAndDetailsTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the CSM IMS image listing enriched with the CFS
//! configuration and BOS-template references each image was produced
//! by / is used in. Ochami uses the trait default and returns
//! [`Error::Message`] ("not implemented for this backend").

use super::*;

impl GetImagesAndDetailsTrait for StaticBackendDispatcher {
  /// Return up to `limit_number` images visible to
  /// `group_group_name_vec`, optionally narrowed by `id_opt`. Each
  /// tuple is `(image, cfs_configuration_name, bos_template_name,
  /// is_bootable)`; the strings are empty when the corresponding
  /// derivative is absent.
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
