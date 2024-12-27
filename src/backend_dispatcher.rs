use std::collections::HashMap;

/// This is the static backend dispatcher
/// To add a new backend:
/// # Add new backend to the StaticBackendDispatcher enum
/// # Add new backend_type to the StaticBackendDispatcher (new) constructor
/// # Add new backend to existing methods in BackendTrait implementation
///
/// To add new functionalities:
/// # Implement new functionalities to BackendTrait implementation
/// NOTE: we assume functionalities are already added to the BackendTrait in 'backend' crate
use backend_dispatcher::{
    contracts::BackendTrait,
    error::Error,
    types::{BootParameters, HsmGroup},
};

pub enum StaticBackendDispatcher {
    CSM(Csm),
    OCHAMI(Ochami),
}

use StaticBackendDispatcher::*;

use mesa::backend_connector::Csm;
use ochami_rs::backend_connector::Ochami;
use serde_json::Value;

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

impl BackendTrait for StaticBackendDispatcher {
    fn test_backend_trait(&self) -> String {
        println!("in manta backend");
        "in manta backend".to_string()
    }

    // AUTHENTICATION
    async fn get_api_token(&self, site_name: &str) -> Result<String, Error> {
        match self {
            CSM(b) => b.get_api_token(site_name).await,
            OCHAMI(b) => b.get_api_token(site_name).await,
        }
    }

    // HSM/GROUP
    async fn get_hsm_name_available(&self, jwt_token: &str) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => b.get_hsm_name_available(jwt_token).await,
            OCHAMI(b) => b.get_hsm_name_available(jwt_token).await,
        }
    }

    // HSM/GROUP
    // FIXME: rename function to 'get_hsm_group_members'
    async fn get_member_vec_from_hsm_name_vec(
        &self,
        auth_token: &str,
        hsm_group_name_vec: Vec<String>,
    ) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => {
                b.get_member_vec_from_hsm_name_vec(auth_token, hsm_group_name_vec)
                    .await
            }
            OCHAMI(b) => {
                b.get_member_vec_from_hsm_name_vec(auth_token, hsm_group_name_vec)
                    .await
            }
        }
    }

    // HSM/GROUP
    async fn get_hsm_map_and_filter_by_hsm_name_vec(
        &self,
        auth_token: &str,
        hsm_name_vec: Vec<&str>,
    ) -> Result<HashMap<String, Vec<String>>, Error> {
        match self {
            CSM(b) => {
                b.get_hsm_map_and_filter_by_hsm_name_vec(auth_token, hsm_name_vec)
                    .await
            }
            OCHAMI(b) => {
                b.get_hsm_map_and_filter_by_hsm_name_vec(auth_token, hsm_name_vec)
                    .await
            }
        }
    }

    // HSM/GROUP
    async fn get_all_hsm_group(&self, auth_token: &str) -> Result<Vec<HsmGroup>, Error> {
        match self {
            CSM(b) => b.get_all_hsm_group(auth_token).await,
            OCHAMI(b) => b.get_all_hsm_group(auth_token).await,
        }
    }

    async fn get_hsm_group(&self, auth_token: &str, hsm_name: &str) -> Result<HsmGroup, Error> {
        match self {
            CSM(b) => b.get_hsm_group(auth_token, hsm_name).await,
            OCHAMI(b) => b.get_hsm_group(auth_token, hsm_name).await,
        }
    }

    async fn add_hsm_group(
        &self,
        auth_token: &str,
        hsm_group: HsmGroup,
    ) -> Result<HsmGroup, Error> {
        match self {
            CSM(b) => b.add_hsm_group(auth_token, hsm_group).await,
            OCHAMI(b) => b.add_hsm_group(auth_token, hsm_group).await,
        }
    }

    async fn delete_hsm_group(
        &self,
        auth_token: &str,
        hsm_group_label: &str,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => b.delete_hsm_group(auth_token, hsm_group_label).await,
            OCHAMI(b) => b.delete_hsm_group(auth_token, hsm_group_label).await,
        }
    }

    // PCS
    async fn power_on_sync(&self, auth_token: &str, nodes: &[String]) -> Result<Value, Error> {
        match self {
            CSM(b) => b.power_on_sync(auth_token, nodes).await,
            OCHAMI(b) => b.power_on_sync(auth_token, nodes).await,
        }
    }

    // PCS
    async fn power_off_sync(
        &self,
        auth_token: &str,
        nodes: &[String],
        force: bool,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => b.power_off_sync(auth_token, nodes, force).await,
            OCHAMI(b) => b.power_off_sync(auth_token, nodes, force).await,
        }
    }

    // PCS
    async fn power_reset_sync(
        &self,
        auth_token: &str,
        nodes: &[String],
        force: bool,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => b.power_reset_sync(auth_token, nodes, force).await,
            OCHAMI(b) => b.power_reset_sync(auth_token, nodes, force).await,
        }
    }

    // BSS/BOOTPARAMETERS
    async fn get_bootparameters(
        &self,
        auth_token: &str,
        nodes: &[String],
    ) -> Result<Vec<BootParameters>, Error> {
        match self {
            CSM(b) => b.get_bootparameters(auth_token, nodes).await,
            OCHAMI(b) => b.get_bootparameters(auth_token, nodes).await,
        }
    }

    // BSS/BOOTPARAMETERS
    async fn update_bootparameters(
        &self,
        auth_token: &str,
        boot_parameters: &BootParameters,
    ) -> Result<(), Error> {
        match self {
            CSM(b) => b.update_bootparameters(auth_token, boot_parameters).await,
            OCHAMI(b) => b.update_bootparameters(auth_token, boot_parameters).await,
        }
    }
}
