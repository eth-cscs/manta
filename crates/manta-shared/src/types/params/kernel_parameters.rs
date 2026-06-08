//! Parameters for `GET /kernel-parameters`.
//!
//! The internal `KernelParamOperation` enum used by the server's
//! kernel-parameter orchestration is not exposed here — it lives in
//! `crate::service::kernel_parameters` because it carries operational logic
//! (mutate, handles_sbps_images) rather than wire data.

/// Typed parameters for fetching kernel boot parameters.
pub struct GetKernelParametersParams {
  /// Group whose members' kernel parameters should be returned.
  pub group_name: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `group_name`.
  pub nodes: Option<String>,
  /// Operator default from `cli.toml`'s `parent_group_group`, used
  /// when neither `group_name` nor `nodes` is supplied.
  pub settings_group_name: Option<String>,
}
