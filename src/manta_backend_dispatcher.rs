use csm_rs::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;

#[derive(Clone)]
pub enum StaticBackendDispatcher {
  CSM(Csm),
  OCHAMI(Ochami),
}

impl StaticBackendDispatcher {
  pub fn new(backend_type: &str, base_url: &str, root_cert: &[u8]) -> Self {
    let csm = Csm::new(base_url, root_cert);
    let ochami = Ochami::new(base_url, root_cert);

    match backend_type {
      "csm" => Self::CSM(csm).into(),
      "ochami" => Self::OCHAMI(ochami).into(),
      _ => {
        eprintln!("ERROR - Backend '{}' not supported", backend_type);
        std::process::exit(1);
      }
    }
  }
}
