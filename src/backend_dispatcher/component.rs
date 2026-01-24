use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::component::ComponentTrait,
  types::{Component, ComponentArrayPostArray, NodeMetadataArray},
};

use StaticBackendDispatcher::*;

use serde_json::Value;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ComponentTrait for StaticBackendDispatcher {
  async fn get_all_nodes(
    &self,
    auth_token: &str,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    match self {
      CSM(b) => b.get_all_nodes(auth_token, nid_only).await,
      OCHAMI(b) => b.get_all_nodes(auth_token, nid_only).await,
    }
  }

  async fn get_node_metadata_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Component>, Error> {
    match self {
      CSM(b) => b.get_node_metadata_available(auth_token).await,
      OCHAMI(b) => b.get_node_metadata_available(auth_token).await,
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
  ) -> Result<NodeMetadataArray, Error> {
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

  async fn delete_node(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.delete_node(auth_token, id).await,
      OCHAMI(b) => b.delete_node(auth_token, id).await,
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
