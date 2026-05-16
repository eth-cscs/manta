//! Parameters for `POST /power`.

/// The power operation to apply to a list of xnames.
#[derive(Debug, Clone, Copy)]
pub enum PowerAction {
  On,
  Off,
  Reset,
}

/// Typed parameters for the power-action service call.
pub struct ApplyPowerParams {
  pub action: PowerAction,
  pub xnames: Vec<String>,
  pub force: bool,
}
