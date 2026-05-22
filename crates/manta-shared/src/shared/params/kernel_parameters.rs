//! Parameters for `GET /kernel-parameters`.
//!
//! The internal `KernelParamOperation` enum used by the server's
//! kernel-parameter orchestration is not exposed here — it lives in
//! `crate::service::kernel_parameters` because it carries operational logic
//! (mutate, handles_sbps_images) rather than wire data.

/// Typed parameters for fetching kernel boot parameters.
pub struct GetKernelParametersParams {
  /// HSM group whose members' kernel parameters should be returned.
  pub hsm_group: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `hsm_group`.
  pub nodes: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used
  /// when neither `hsm_group` nor `nodes` is supplied.
  pub settings_hsm_group_name: Option<String>,
}
