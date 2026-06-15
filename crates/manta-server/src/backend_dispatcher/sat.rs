//! `SatTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl SatTrait for StaticBackendDispatcher {
  async fn apply_sat_file(
    &self,
    params: ApplySatFileParams<'_>,
  ) -> Result<
    (
      Vec<CfsConfigurationResponse>,
      Vec<Image>,
      Vec<BosSessionTemplate>,
      Vec<BosSession>,
    ),
    Error,
  > {
    dispatch!(self, apply_sat_file, params)
  }

  async fn validate_sat_file(
    &self,
    params: ValidateSatFileParams<'_>,
  ) -> Result<(), Error> {
    dispatch!(self, validate_sat_file, params)
  }

  async fn apply_configuration(
    &self,
    params: ApplyConfigurationParams<'_>,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(self, apply_configuration, params)
  }

  async fn apply_image(
    &self,
    params: ApplyImageParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_image, params)
  }

  async fn apply_sat_image_create_session(
    &self,
    params: ApplyImageCreateSessionParams<'_>,
  ) -> Result<CfsSessionGetResponse, Error> {
    dispatch!(self, apply_sat_image_create_session, params)
  }

  async fn apply_sat_image_stamp_from_session(
    &self,
    params: ApplyImageStampParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_sat_image_stamp_from_session, params)
  }

  async fn apply_session_template(
    &self,
    params: ApplySessionTemplateParams<'_>,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    dispatch!(self, apply_session_template, params)
  }
}
