//! Parameters for `GET /kernel-parameters`.
//!
//! The internal `KernelParamOperation` enum used by the server's
//! kernel-parameter orchestration is not exposed here — it lives in
//! `crate::service::kernel_parameters` because it carries operational logic
//! (mutate, handles_sbps_images) rather than wire data.

/// Typed parameters for fetching kernel boot parameters.
pub struct GetKernelParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}
