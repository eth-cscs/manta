use std::collections::HashMap;

use crate::common::audit::Auditor;

use manta_backend_dispatcher::types::K8sDetails;
use serde::{Deserialize, Serialize};

/* #[derive(Serialize, Deserialize, Debug)]
pub enum K8sAuth {
    #[serde(rename = "native")]
    Native {
        certificate_authority_data: String,
        client_certificate_data: String,
        client_key_data: String,
    },
    #[serde(rename = "vault")]
    Vault {
        base_url: String,
        // secret_path: String,
        // role_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct K8sDetails {
    pub api_url: String,
    pub authentication: K8sAuth,
} */

/* #[derive(Serialize, Deserialize, Debug)]
pub struct Kafka {
    pub brokers: Vec<String>,
    pub topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SysLog {
    pub server: String,
    pub port: u16,
} */

/* #[derive(Serialize, Deserialize, Debug)]
pub struct Audit {
    pub kafka: Option<Kafka>,
    pub syslog: Option<SysLog>,
} */

#[derive(Serialize, Deserialize, Debug)]
pub struct Site {
  pub backend: String,
  pub socks5_proxy: Option<String>,
  pub shasta_base_url: String,
  pub k8s: Option<K8sDetails>,
  // pub k8s_api_url: Option<String>,
  pub vault_base_url: Option<String>,
  pub vault_secret_path: Option<String>,
  // pub vault_role_id: Option<String>,
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
