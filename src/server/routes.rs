use std::sync::Arc;

use axum::{
  routing::{delete, get, post},
  Router,
};

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
    // --- Write endpoints ---
    // Nodes
    .route("/nodes", post(handlers::add_node))
    .route("/nodes/{id}", delete(handlers::delete_node))
    // Groups
    .route("/groups", post(handlers::create_group))
    .route("/groups/{label}", delete(handlers::delete_group))
    .route(
      "/groups/{name}/members",
      post(handlers::add_nodes_to_group)
        .delete(handlers::delete_group_members),
    )
    // Boot parameters
    .route(
      "/boot-parameters",
      post(handlers::add_boot_parameters)
        .put(handlers::update_boot_parameters)
        .delete(handlers::delete_boot_parameters),
    )
    // Redfish endpoints
    .route(
      "/redfish-endpoints",
      post(handlers::add_redfish_endpoint)
        .put(handlers::update_redfish_endpoint),
    )
    .route(
      "/redfish-endpoints/{id}",
      delete(handlers::delete_redfish_endpoint),
    )
    // Sessions (delete with dry_run)
    .route("/sessions/{name}", delete(handlers::delete_session))
    // Sessions (create)
    .route("/sessions", post(handlers::create_session))
    // Images (delete with dry_run)
    .route("/images", delete(handlers::delete_images))
    // Configurations (delete with dry_run)
    .route(
      "/configurations",
      delete(handlers::delete_configurations),
    )
    // Boot config (apply with dry_run)
    .route("/boot-config", post(handlers::apply_boot_config))
    // Kernel parameters (apply with dry_run)
    .route(
      "/kernel-parameters/apply",
      post(handlers::apply_kernel_parameters),
    )
    // Migrate
    .route("/migrate/nodes", post(handlers::migrate_nodes))
    .route("/migrate/backup", post(handlers::migrate_backup))
    .route("/migrate/restore", post(handlers::migrate_restore))
    // Ephemeral environment
    .route("/ephemeral-env", post(handlers::create_ephemeral_env))
    // Power management
    .route("/power", post(handlers::post_power))
    // BOS session from template
    .route("/templates/{name}/sessions", post(handlers::post_template_session))
    // CFS session logs (SSE)
    .route("/sessions/{name}/logs", get(handlers::get_session_logs))
    // SAT file apply
    .route("/sat-file", post(handlers::post_sat_file))
    // Health check
    .route("/health", get(handlers::health))
    // Kernel parameters — add and delete
    .route("/kernel-parameters/add", post(handlers::add_kernel_parameters))
    .route(
      "/kernel-parameters",
      delete(handlers::delete_kernel_parameters),
    )
    // Hardware cluster member management
    .route(
      "/hardware-clusters/{target}/members",
      post(handlers::add_hw_component)
        .delete(handlers::delete_hw_component),
    )
    // Hardware cluster configuration (pin/unpin)
    .route(
      "/hardware-clusters/{target}/configuration",
      post(handlers::apply_hw_configuration),
    )
    // CFS session from pre-resolved repos with HSM validation
    .route("/sessions/apply", post(handlers::apply_session))
    // WebSocket consoles
    .route("/nodes/{xname}/console", get(handlers::console_node_ws))
    .route("/sessions/{name}/console", get(handlers::console_session_ws));

  Router::new().nest("/api/v1", api).with_state(state)
}
