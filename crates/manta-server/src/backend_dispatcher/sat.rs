//! [`SatTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards the SAT-file (Site Activation Template) orchestration —
//! the per-section helpers that the `manta sat apply` pipeline calls
//! one entry at a time, plus the all-in-one [`SatTrait::apply_sat_file`]
//! entry point. The CSM backend implements each method by composing
//! CFS, IMS, and BOS calls; Ochami's `impl SatTrait for Ochami {}` is
//! empty, so every method on the Ochami branch returns
//! [`Error::Message`] from the trait default ("not implemented for
//! this backend").

use super::*;

impl SatTrait for StaticBackendDispatcher {
  /// Apply an entire SAT file (`configurations`, `images`,
  /// `session_templates`, optional `sessions`) in one call. Returns
  /// the tuple `(configurations, images, session_templates, sessions)`
  /// of what was created — empty vectors when a section is absent.
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

  /// Pre-flight check: validate `params` against live backend state
  /// without mutating anything. Returns `Ok(())` if the SAT file
  /// would apply cleanly, fail-fast on the first detected mismatch.
  async fn validate_sat_file(
    &self,
    params: ValidateSatFileParams<'_>,
  ) -> Result<(), Error> {
    dispatch!(self, validate_sat_file, params)
  }

  /// Apply a single SAT `configurations[]` entry — resolve any
  /// `product:`/`git:` layers, POST the resulting
  /// `CfsConfigurationRequest` to CFS, return the persisted
  /// configuration (or a mock in `params.dry_run`).
  async fn apply_configuration(
    &self,
    params: ApplyConfigurationParams<'_>,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(self, apply_configuration, params)
  }

  /// Apply a single SAT `images[]` entry synchronously: kick off the
  /// CFS image-build session and wait for it to complete, stamping
  /// `manta.image_session.*` provenance metadata onto the produced
  /// IMS image before returning. For non-blocking flow, drive
  /// [`apply_sat_image_create_session`](Self::apply_sat_image_create_session)
  /// and
  /// [`apply_sat_image_stamp_from_session`](Self::apply_sat_image_stamp_from_session)
  /// directly.
  async fn apply_image(
    &self,
    params: ApplyImageParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_image, params)
  }

  /// First half of the split image-apply flow: validate, resolve
  /// `base.image_ref`, POST the CFS image-build session. Returns the
  /// created session record without waiting for it to finish.
  async fn apply_sat_image_create_session(
    &self,
    params: ApplyImageCreateSessionParams<'_>,
  ) -> Result<CfsSessionGetResponse, Error> {
    dispatch!(self, apply_sat_image_create_session, params)
  }

  /// Second half of the split image-apply flow: read the (assumed
  /// completed) CFS session referenced by `params`, locate the IMS
  /// image it produced, stamp `manta.image_session.*` provenance
  /// metadata, and return the stamped image.
  async fn apply_sat_image_stamp_from_session(
    &self,
    params: ApplyImageStampParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_sat_image_stamp_from_session, params)
  }

  /// Apply a single SAT `session_templates[]` entry — PUT the BOS
  /// session template, then (when `params.reboot` is true) POST a
  /// BOS session derived from it. Returns
  /// `(persisted_template, Some(session))` when a reboot session was
  /// kicked off, `(persisted_template, None)` otherwise.
  async fn apply_session_template(
    &self,
    params: ApplySessionTemplateParams<'_>,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    dispatch!(self, apply_session_template, params)
  }
}
