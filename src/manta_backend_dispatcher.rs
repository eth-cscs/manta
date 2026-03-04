use anyhow::{Error, bail};
use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum StaticBackendDispatcher {
  CSM(Csm),
  OCHAMI(Ochami),
}

impl StaticBackendDispatcher {
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
