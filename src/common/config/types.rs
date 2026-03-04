use std::collections::HashMap;

use crate::common::audit::Auditor;

use manta_backend_dispatcher::types::K8sDetails;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Site {
  pub backend: String,
  pub socks5_proxy: Option<String>,
  pub shasta_base_url: String,
  pub k8s: Option<K8sDetails>,
  pub vault_base_url: Option<String>,
  pub vault_secret_path: Option<String>,
  pub root_ca_cert_file: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MantaConfiguration {
  pub log: String,
  pub site: String,
  pub parent_hsm_group: String,
  pub audit_file: String,
  pub sites: HashMap<String, Site>,
  pub auditor: Option<Auditor>,
}
