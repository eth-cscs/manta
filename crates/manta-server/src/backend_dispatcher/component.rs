//! [`ComponentTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the HSM (Hardware State Manager)
//! `/apis/smd/hsm/v2/State/Components` API. Both CSM and Ochami
//! implement this trait natively.

use super::*;

impl ComponentTrait for StaticBackendDispatcher {
  /// `GET /State/Components?type=Node` — every node in HSM. When
  /// `nid_only` is set, the backend forwards `nid_only=true` to the
  /// HSM which limits the returned representation to the NID field.
  async fn get_all_nodes(
    &self,
    auth_token: &str,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    dispatch!(self, get_all_nodes, auth_token, nid_only)
  }

  /// RBAC-aware node listing: HSM filtered to the components the
  /// caller's groups grant access to. Used to populate the user's
  /// visible inventory after auth.
  async fn get_node_metadata_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Component>, Error> {
    dispatch!(self, get_node_metadata_available, auth_token)
  }

  /// `GET /State/Components` with the full HSM filter set forwarded
  /// verbatim. Every `Option<&str>` maps 1:1 to a query parameter on
  /// the upstream endpoint; the `*_only` parameters restrict the
  /// returned fields (HSM's "lite" projections).
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
    dispatch!(
      self,
      get,
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
      nid_only
    )
  }

  /// `POST /State/Components` — bulk-create node components.
  async fn post_nodes(
    &self,
    auth_token: &str,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    dispatch!(self, post_nodes, auth_token, component)
  }

  /// `DELETE /State/Components/{id}` — remove a single component
  /// (typically an xname). The returned `HsmActionResponse` reports
  /// how many records were affected.
  async fn delete_node(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_node, auth_token, id)
  }

  /// Resolve a NID expression (single id, range, or regex when
  /// `is_regex`) to xnames by scanning HSM nodes. The matching logic
  /// runs client-side because HSM's NID query parameter does not
  /// support ranges or patterns.
  async fn nid_to_xname(
    &self,
    auth_token: &str,
    user_input_nid: &str,
    is_regex: bool,
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, nid_to_xname, auth_token, user_input_nid, is_regex)
  }
}
