use anyhow::{Error, bail};
use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
/// Routes API calls to either a CSM or OCHAMI backend.
///
/// All backend-specific trait methods are dispatched via
/// the [`dispatch!`] macro defined in the `backend_dispatcher`
/// module.
pub enum StaticBackendDispatcher {
  CSM(Csm),
  OCHAMI(Ochami),
}

impl StaticBackendDispatcher {
  /// Create a new dispatcher for the given backend type.
  ///
  /// `backend_type` must be `"csm"` or `"ochami"`;
  /// any other value returns an error.
  pub fn new(
    backend_type: &str,
    base_url: &str,
    root_cert: &[u8],
  ) -> Result<Self, Error> {
    let csm = Csm::new(base_url, root_cert);
    let ochami = Ochami::new(base_url, root_cert);

    match backend_type {
      "csm" => Ok(Self::CSM(csm)),
      "ochami" => Ok(Self::OCHAMI(ochami)),
      _ => bail!("Backend '{}' not supported", backend_type),
    }
  }
}
