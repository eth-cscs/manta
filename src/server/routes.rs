use std::sync::Arc;

use axum::{routing::get, Router};

use super::handlers;
use super::ServerState;

/// Build the axum router with all API endpoints.
pub fn build_router(state: Arc<ServerState>) -> Router {
  let api = Router::new()
    // --- GET endpoints ---
    .route("/sessions", get(handlers::get_sessions))
    .route("/configurations", get(handlers::get_configurations))
    .route("/nodes", get(handlers::get_nodes))
    .route("/groups", get(handlers::get_groups))
    .route("/images", get(handlers::get_images))
    .route("/templates", get(handlers::get_templates))
    .route("/boot-parameters", get(handlers::get_boot_parameters))
    .route("/kernel-parameters", get(handlers::get_kernel_parameters))
    .route("/redfish-endpoints", get(handlers::get_redfish_endpoints))
    .route("/clusters", get(handlers::get_clusters))
    .route("/hardware-clusters", get(handlers::get_hardware_clusters))
    .route("/hardware-nodes", get(handlers::get_hardware_nodes))
    // Health check
    .route("/health", get(handlers::health));

  Router::new().nest("/api/v1", api).with_state(state)
}
