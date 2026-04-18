use std::collections::HashMap;

use crate::common::audit::Auditor;

use manta_backend_dispatcher::types::K8sDetails;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
/// Connection details for a single ALPS site (CSM or
/// OCHAMI instance).
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
/// Top-level manta configuration, persisted as TOML
/// under `~/.config/manta/config.toml`.
pub struct MantaConfiguration {
  pub log: String,
  pub site: String,
  pub parent_hsm_group: String,
  pub audit_file: String,
  pub sites: HashMap<String, Site>,
  pub auditor: Option<Auditor>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::K8sAuth;

  fn make_minimal_config() -> MantaConfiguration {
    let site = Site {
      backend: "csm".to_string(),
      socks5_proxy: None,
      shasta_base_url: "https://api.example.com".to_string(),
      k8s: None,
      vault_base_url: None,
      vault_secret_path: None,
      root_ca_cert_file: "cert.pem".to_string(),
    };

    let mut sites = HashMap::new();
    sites.insert("alps".to_string(), site);

    MantaConfiguration {
      log: "error".to_string(),
      site: "alps".to_string(),
      parent_hsm_group: String::new(),
      audit_file: "/tmp/manta.log".to_string(),
      sites,
      auditor: None,
    }
  }

  #[test]
  fn config_roundtrip_toml_minimal() {
    let config = make_minimal_config();
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: MantaConfiguration = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.log, "error");
    assert_eq!(parsed.site, "alps");
    assert!(parsed.auditor.is_none());
    let site = parsed.sites.get("alps").unwrap();
    assert_eq!(site.backend, "csm");
    assert_eq!(site.shasta_base_url, "https://api.example.com");
    assert!(site.socks5_proxy.is_none());
    assert!(site.k8s.is_none());
  }

  #[test]
  fn config_roundtrip_toml_with_k8s() {
    let mut config = make_minimal_config();
    let site = config.sites.get_mut("alps").unwrap();
    site.k8s = Some(K8sDetails {
      api_url: "https://10.0.0.1:6443".to_string(),
      authentication: K8sAuth::Native {
        certificate_authority_data: "ca-data".to_string(),
        client_certificate_data: "client-cert".to_string(),
        client_key_data: "client-key".to_string(),
      },
    });

    let toml_str = toml::to_string(&config).unwrap();
    let parsed: MantaConfiguration = toml::from_str(&toml_str).unwrap();

    let k8s = parsed.sites.get("alps").unwrap().k8s.as_ref().unwrap();
    assert_eq!(k8s.api_url, "https://10.0.0.1:6443");
    match &k8s.authentication {
      K8sAuth::Native {
        certificate_authority_data,
        ..
      } => assert_eq!(certificate_authority_data, "ca-data"),
      _ => panic!("Expected Native K8sAuth"),
    }
  }

  #[test]
  fn config_roundtrip_toml_with_socks5_proxy() {
    let mut config = make_minimal_config();
    config.sites.get_mut("alps").unwrap().socks5_proxy =
      Some("socks5h://127.0.0.1:1080".to_string());

    let toml_str = toml::to_string(&config).unwrap();
    let parsed: MantaConfiguration = toml::from_str(&toml_str).unwrap();

    assert_eq!(
      parsed.sites.get("alps").unwrap().socks5_proxy.as_deref(),
      Some("socks5h://127.0.0.1:1080")
    );
  }

  #[test]
  fn config_roundtrip_toml_with_vault() {
    let mut config = make_minimal_config();
    let site = config.sites.get_mut("alps").unwrap();
    site.vault_base_url = Some("https://vault.example.com:8200".to_string());
    site.vault_secret_path = Some("secret/shasta".to_string());

    let toml_str = toml::to_string(&config).unwrap();
    let parsed: MantaConfiguration = toml::from_str(&toml_str).unwrap();

    let site = parsed.sites.get("alps").unwrap();
    assert_eq!(
      site.vault_base_url.as_deref(),
      Some("https://vault.example.com:8200")
    );
    assert_eq!(site.vault_secret_path.as_deref(), Some("secret/shasta"));
  }

  #[test]
  fn config_multiple_sites() {
    let mut config = make_minimal_config();
    config.sites.insert(
      "eiger".to_string(),
      Site {
        backend: "ochami".to_string(),
        socks5_proxy: None,
        shasta_base_url: "https://api.eiger.example.com".to_string(),
        k8s: None,
        vault_base_url: None,
        vault_secret_path: None,
        root_ca_cert_file: "eiger_cert.pem".to_string(),
      },
    );

    let toml_str = toml::to_string(&config).unwrap();
    let parsed: MantaConfiguration = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.sites.len(), 2);
    assert_eq!(parsed.sites.get("eiger").unwrap().backend, "ochami");
  }

  #[test]
  fn config_deserialize_missing_required_field_fails() {
    let bad_toml = r#"
      log = "error"
      site = "alps"
      # missing parent_hsm_group, audit_file, sites
    "#;
    let result = toml::from_str::<MantaConfiguration>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn config_deserialize_invalid_toml_fails() {
    let bad_toml = "this is not valid toml {{{}}}";
    let result = toml::from_str::<MantaConfiguration>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn site_deserialize_missing_backend_fails() {
    let bad_toml = r#"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"
      # missing backend
    "#;
    let result = toml::from_str::<Site>(bad_toml);
    assert!(result.is_err());
  }
}
