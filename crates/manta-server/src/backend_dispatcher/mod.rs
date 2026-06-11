//! `StaticBackendDispatcher` trait implementations.
//!
//! Each backend trait gets its own sibling file; this file owns the
//! shared imports and the `dispatch!` macro. Sibling files do
//! `use super::*;` to pick up the items; the macro is textually
//! scoped so it's visible to every `mod` declaration that follows it.
//!
//! Without explicit forwards the calls would fall through to the
//! `SatTrait` (etc.) default "not implemented" impls in the
//! `manta-backend-dispatcher` crate.

use std::collections::HashMap;
use std::pin::Pin;

use chrono::NaiveDateTime;
use futures::AsyncBufRead;
use serde_json::Value;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_hw_cluster_pin::ApplyHwClusterPin;
use manta_backend_dispatcher::interfaces::apply_sat_file::{
  ApplyConfigurationParams, ApplyImageCreateSessionParams, ApplyImageParams,
  ApplyImageStampParams, ApplySatFileParams, ApplySessionTemplateParams,
  SatTrait,
};
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::authentication::AuthenticationTrait;
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::console::ConsoleTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::interfaces::ims::{
  GetImagesAndDetailsTrait, ImsTrait,
};
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;
use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::{
  self, Component, ComponentArrayPostArray, Group, HWInventory,
  HWInventoryByLocationList, HsmActionResponse, K8sDetails, NodeMetadataArray,
  NodeSummary,
};
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_details::LayerDetails;
use manta_backend_dispatcher::types::cfs::cfs_configuration_request::CfsConfigurationRequest;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
  CfsConfigurationResponse, Layer,
};
use manta_backend_dispatcher::types::cfs::component::Component as CfsComponent;
use manta_backend_dispatcher::types::cfs::session::{
  CfsSessionGetResponse, CfsSessionPostRequest,
};
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};
use manta_backend_dispatcher::types::ims::{Image, PatchImage};
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};

use crate::dispatcher::StaticBackendDispatcher;
use StaticBackendDispatcher::*;

/// Dispatches a method call to the underlying backend variant.
///
/// Both `CSM` and `OCHAMI` variants always delegate to the same method
/// with identical arguments, so this macro eliminates the repetitive
/// `match self` boilerplate.
///
/// # Usage
/// ```ignore
/// // async method:
/// dispatch!(self, method_name, arg1, arg2)
/// // sync method:
/// dispatch!(sync self, method_name, arg1)
/// ```
macro_rules! dispatch {
  // async (default): adds `.await` after the call
  ($self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*).await,
      OCHAMI(b) => b.$method($($arg),*).await,
    }
  };
  // sync: no `.await`
  (sync $self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*),
      OCHAMI(b) => b.$method($($arg),*),
    }
  };
}

mod apply_hw_cluster_pin;
mod apply_session;
mod authentication;
mod boot_parameters;
mod cfs;
mod cluster_session;
mod cluster_template;
mod component;
mod console;
mod delete_configurations;
mod get_images;
mod group;
mod hardware_inventory;
mod ims;
mod migrate_backup;
mod migrate_restore;
mod pcs;
mod redfish_endpoint;
mod sat;
