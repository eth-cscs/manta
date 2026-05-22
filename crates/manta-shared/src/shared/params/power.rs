//! Parameters for `POST /power`.

/// The power operation to apply to a list of xnames.
#[derive(Debug, Clone, Copy)]
pub enum PowerAction {
  /// Power on (cold start) the listed xnames.
  On,
  /// Power off the listed xnames; graceful unless `force` is set.
  Off,
  /// Power-cycle (reset) the listed xnames; graceful unless `force`
  /// is set.
  Reset,
}

/// Typed parameters for the power-action service call.
pub struct ApplyPowerParams {
  /// Power operation to perform on every entry in `xnames`.
  pub action: PowerAction,
  /// Resolved list of xnames (already expanded from any HSM-group
  /// or hostlist expression by the caller).
  pub xnames: Vec<String>,
  /// When true, perform a hard power off / reset without the
  /// graceful shutdown path.
  pub force: bool,
}
