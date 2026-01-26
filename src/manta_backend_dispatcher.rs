use anyhow::Error;
use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;

#[derive(Clone)]
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
      "csm" => Ok(Self::CSM(csm).into()),
      "ochami" => Ok(Self::OCHAMI(ochami).into()),
      _ => Err(Error::msg(format!(
        "Backend '{}' not supported",
        backend_type
      ))),
    }
  }
}
