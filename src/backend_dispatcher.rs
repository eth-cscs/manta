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
    interfaces::{group::GroupTrait, hardware_metadata::HardwareMetadataTrait},
    types::{
        BootParameters, ComponentArrayPostArray, Group, HWInventoryByLocationList,
        HardwareMetadataArray,
    },
};

#[derive(Clone)]
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

impl GroupTrait for StaticBackendDispatcher {
    async fn get_group_available(&self, auth_token: &str) -> Result<Vec<Group>, Error> {
        match self {
            CSM(b) => b.get_group_available(auth_token).await,
            OCHAMI(b) => b.get_group_available(auth_token).await,
        }
    }

    async fn add_group(&self, auth_token: &str, hsm_group: Group) -> Result<Group, Error> {
        match self {
            CSM(b) => b.add_group(auth_token, hsm_group).await,
            OCHAMI(b) => b.add_group(auth_token, hsm_group).await,
        }
    }

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
}

impl HardwareMetadataTrait for StaticBackendDispatcher {
    async fn get_all_nodes(
        &self,
        auth_token: &str,
        nid_only: Option<&str>,
    ) -> Result<HardwareMetadataArray, Error> {
        match self {
            CSM(b) => b.get_all_nodes(auth_token, nid_only).await,
            OCHAMI(b) => b.get_all_nodes(auth_token, nid_only).await,
        }
    }

    async fn get(
        &self,
        auth_token: &str,
        id: Option<&str>,
        r#type: Option<&str>,
        state: Option<&str>,
        flag: Option<&str>,
        role: Option<&str>,
        subrole: Option<&str>,
        enabled: Option<&str>,
        software_status: Option<&str>,
        subtype: Option<&str>,
        arch: Option<&str>,
        class: Option<&str>,
        nid: Option<&str>,
        nid_start: Option<&str>,
        nid_end: Option<&str>,
        partition: Option<&str>,
        group: Option<&str>,
        state_only: Option<&str>,
        flag_only: Option<&str>,
        role_only: Option<&str>,
        nid_only: Option<&str>,
    ) -> Result<HardwareMetadataArray, Error> {
        match self {
            CSM(b) => {
                b.get(
                    auth_token,
                    id,
                    r#type,
                    state,
                    flag,
                    role,
                    subrole,
                    enabled,
                    software_status,
                    subtype,
                    arch,
                    class,
                    nid,
                    nid_start,
                    nid_end,
                    partition,
                    group,
                    state_only,
                    flag_only,
                    role_only,
                    nid_only,
                )
                .await
            }
            OCHAMI(b) => {
                b.get(
                    auth_token,
                    id,
                    r#type,
                    state,
                    flag,
                    role,
                    subrole,
                    enabled,
                    software_status,
                    subtype,
                    arch,
                    class,
                    nid,
                    nid_start,
                    nid_end,
                    partition,
                    group,
                    state_only,
                    flag_only,
                    role_only,
                    nid_only,
                )
                .await
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

    async fn post_member(
        &self,
        auth_token: &str,
        group_label: &str,
        xname: &str,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => b.post_member(auth_token, group_label, xname).await,
            OCHAMI(b) => b.post_member(auth_token, group_label, xname).await,
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

    async fn delete_group(&self, auth_token: &str, hsm_group_label: &str) -> Result<Value, Error> {
        match self {
            CSM(b) => b.delete_group(auth_token, hsm_group_label).await,
            OCHAMI(b) => b.delete_group(auth_token, hsm_group_label).await,
        }
    }

    // HSM/INVENTORY/HARDWARE
    async fn get_inventory_hardware_query(
        &self,
        auth_token: &str,
        xname: &str,
        r#type: Option<&str>,
        children: Option<bool>,
        parents: Option<bool>,
        partition: Option<&str>,
        format: Option<&str>,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => {
                b.get_inventory_hardware_query(
                    auth_token, xname, r#type, children, parents, partition, format,
                )
                .await
            }
            OCHAMI(b) => {
                b.get_inventory_hardware_query(
                    auth_token, xname, r#type, children, parents, partition, format,
                )
                .await
            }
        }
    }

    // HSM/INVENTORY/HARDWARE
    async fn post_inventory_hardware(
        &self,
        auth_token: &str,
        hardware: HWInventoryByLocationList,
    ) -> Result<Value, Error> {
        match self {
            CSM(b) => b.post_inventory_hardware(auth_token, hardware).await,
            OCHAMI(b) => b.post_inventory_hardware(auth_token, hardware).await,
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

    async fn post_nodes(
        &self,
        auth_token: &str,
        component: ComponentArrayPostArray,
    ) -> Result<(), Error> {
        match self {
            CSM(b) => b.post_nodes(auth_token, component).await,
            OCHAMI(b) => b.post_nodes(auth_token, component).await,
        }
    }

    async fn delete_node(&self, auth_token: &str, id: &str) -> Result<Value, Error> {
        match self {
            CSM(b) => b.delete_node(auth_token, id).await,
            OCHAMI(b) => b.delete_node(auth_token, id).await,
        }
    }

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

    async fn nid_to_xname(
        &self,
        auth_token: &str,
        user_input_nid: &str,
        is_regex: bool,
    ) -> Result<Vec<String>, Error> {
        match self {
            CSM(b) => b.nid_to_xname(auth_token, user_input_nid, is_regex).await,
            OCHAMI(b) => b.nid_to_xname(auth_token, user_input_nid, is_regex).await,
        }
    }
}
