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
    types::{BootParameters, Group},
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
    async fn get_group_name_available(&self, jwt_token: &str) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => b.get_group_name_available(jwt_token).await,
            OCHAMI(b) => b.get_group_name_available(jwt_token).await,
        }
    }

    // HSM/GROUP
    // FIXME: rename function to 'get_hsm_group_members'
    async fn get_member_vec_from_group_name_vec(
        &self,
        auth_token: &str,
        hsm_group_name_vec: Vec<String>,
    ) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => {
                b.get_member_vec_from_group_name_vec(auth_token, hsm_group_name_vec)
                    .await
            }
            OCHAMI(b) => {
                b.get_member_vec_from_group_name_vec(auth_token, hsm_group_name_vec)
                    .await
            }
        }
    }

    async fn post_members(
        &self,
        auth_token: &str,
        group_label: &str,
        xnames: &[&str],
    ) -> Result<(), Error> {
        match self {
            CSM(b) => b.post_members(auth_token, group_label, xnames).await,
            OCHAMI(b) => b.post_members(auth_token, group_label, xnames).await,
        }
    }

    async fn add_members_to_group(
        &self,
        auth_token: &str,
        group_label: &str,
        xnames: Vec<&str>,
    ) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => {
                b.add_members_to_group(auth_token, group_label, xnames)
                    .await
            }
            OCHAMI(b) => {
                b.add_members_to_group(auth_token, group_label, xnames)
                    .await
            }
        }
    }

    async fn delete_member_from_group(
        &self,
        auth_token: &str,
        group_label: &str,
        xname: &str,
    ) -> Result<(), Error> {
        match self {
            CSM(b) => {
                b.delete_member_from_group(auth_token, group_label, xname)
                    .await
            }
            OCHAMI(b) => {
                b.delete_member_from_group(auth_token, group_label, xname)
                    .await
            }
        }
    }

    // HSM/GROUP
    async fn migrate_group_members(
        &self,
        auth_token: &str,
        target_hsm_group_name: &str,
        parent_hsm_group_name: &str,
        new_target_hsm_members: Vec<&str>,
    ) -> Result<(Vec<String>, Vec<String>), Error> {
        match self {
            CSM(b) => {
                b.migrate_group_members(
                    auth_token,
                    target_hsm_group_name,
                    parent_hsm_group_name,
                    new_target_hsm_members,
                )
                .await
            }
            OCHAMI(b) => {
                b.migrate_group_members(
                    auth_token,
                    target_hsm_group_name,
                    parent_hsm_group_name,
                    new_target_hsm_members,
                )
                .await
            }
        }
    }

    // HSM/GROUP
    async fn update_group_members(
        &self,
        auth_token: &str,
        group_name: &str,
        members_to_remove: &Vec<String>,
        members_to_add: &Vec<String>,
    ) -> Result<(), Error> {
        match self {
            CSM(b) => {
                b.update_group_members(auth_token, group_name, members_to_remove, members_to_add)
                    .await
            }
            OCHAMI(b) => {
                b.update_group_members(auth_token, group_name, members_to_remove, members_to_add)
                    .await
            }
        }
    }

    // HSM/GROUP
    async fn get_group_map_and_filter_by_group_vec(
        &self,
        auth_token: &str,
        hsm_name_vec: Vec<&str>,
    ) -> Result<HashMap<String, Vec<String>>, Error> {
        match self {
            CSM(b) => {
                b.get_group_map_and_filter_by_group_vec(auth_token, hsm_name_vec)
                    .await
            }
            OCHAMI(b) => {
                b.get_group_map_and_filter_by_group_vec(auth_token, hsm_name_vec)
                    .await
            }
        }
    }

    // HSM/GROUP
    async fn get_all_groups(&self, auth_token: &str) -> Result<Vec<Group>, Error> {
        match self {
            CSM(b) => b.get_all_groups(auth_token).await,
            OCHAMI(b) => b.get_all_groups(auth_token).await,
        }
    }

    async fn get_group(&self, auth_token: &str, hsm_name: &str) -> Result<Group, Error> {
        match self {
            CSM(b) => b.get_group(auth_token, hsm_name).await,
            OCHAMI(b) => b.get_group(auth_token, hsm_name).await,
        }
    }

    async fn add_group(&self, auth_token: &str, hsm_group: Group) -> Result<Group, Error> {
        match self {
            CSM(b) => b.add_group(auth_token, hsm_group).await,
            OCHAMI(b) => b.add_group(auth_token, hsm_group).await,
        }
    }

    async fn delete_group(&self, auth_token: &str, hsm_group_label: &str) -> Result<Value, Error> {
        match self {
            CSM(b) => b.delete_group(auth_token, hsm_group_label).await,
            OCHAMI(b) => b.delete_group(auth_token, hsm_group_label).await,
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
