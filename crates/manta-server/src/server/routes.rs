//! Axum router registration: maps every `/api/v1/` path to its handler.
//!
//! The OpenAPI JSON spec is served at `GET /openapi.json` and the
//! Swagger UI is served at `GET /docs`.

use std::sync::Arc;

use axum::{
  Extension, Router,
  http::StatusCode,
  middleware,
  routing::{delete, get, post},
};
use tower_http::timeout::TimeoutLayer;
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;

use super::ServerState;
use super::api_doc::ApiDoc;
use super::auth_middleware::{
  AuthRateLimiter, rate_limit, strip_body_for_logs,
};
use super::handlers;

/// Build the axum router with all API endpoints and OpenAPI doc routes.
///
/// The single global `request_timeout` is applied to every route as an
/// outer `TimeoutLayer`. `POST /power` now returns immediately with a
/// PCS transition id (the polling loop runs CLI-side), so it fits
/// well under the default timeout — no per-route override is needed.
pub fn build_router(state: Arc<ServerState>) -> Router {
  let api = Router::new()
    // --- GET endpoints ---
    .route("/sessions", get(handlers::get_sessions))
    .route("/configurations", get(handlers::get_configurations))
    .route("/nodes", get(handlers::get_nodes))
    .route("/groups", get(handlers::get_groups))
    .route("/groups/available", get(handlers::get_available_groups))
    .route("/groups/all", get(handlers::get_all_groups))
    .route("/images", get(handlers::get_images))
    .route("/templates", get(handlers::get_templates))
    .route("/boot-parameters", get(handlers::get_boot_parameters))
    .route("/kernel-parameters", get(handlers::get_kernel_parameters))
    .route("/redfish-endpoints", get(handlers::get_redfish_endpoints))
    // Canonical (group-centric) read endpoints
    .route("/groups/nodes", get(handlers::get_groups_nodes))
    .route("/groups/hardware", get(handlers::get_groups_hardware))
    // Deprecated aliases retained for one release. Each handler logs
    // a server-side warning and forwards to the canonical impl.
    .route("/clusters", get(handlers::get_clusters_deprecated))
    .route(
      "/hardware-clusters",
      get(handlers::get_hardware_clusters_deprecated),
    )
    .route(
      "/hardware-nodes-list",
      get(handlers::get_hardware_nodes_list),
    )
    // --- Write endpoints ---
    // Nodes
    .route("/nodes", post(handlers::add_node))
    .route("/nodes/{id}", delete(handlers::delete_node))
    // Groups
    .route("/groups", post(handlers::create_group))
    .route("/groups/{label}", delete(handlers::delete_group))
    .route(
      "/groups/{name}/members",
      post(handlers::add_nodes_to_group).delete(handlers::delete_group_members),
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
    .route("/configurations", delete(handlers::delete_configurations))
    // Boot config (apply with dry_run)
    .route("/boot-config", post(handlers::apply_boot_config))
    // Kernel parameters (apply, add, delete)
    .route(
      "/kernel-parameters/apply",
      post(handlers::apply_kernel_parameters),
    )
    .route(
      "/kernel-parameters/add",
      post(handlers::add_kernel_parameters),
    )
    .route(
      "/kernel-parameters",
      delete(handlers::delete_kernel_parameters),
    )
    // Migrate
    .route("/migrate/nodes", post(handlers::migrate_nodes))
    .route("/migrate/backup", post(handlers::migrate_backup))
    .route("/migrate/restore", post(handlers::migrate_restore))
    // Ephemeral environment
    .route("/ephemeral-env", post(handlers::create_ephemeral_env))
    // Power management — POST starts a PCS transition and returns
    // immediately; GET snapshots the transition for the CLI poll loop.
    .route("/power", post(handlers::post_power))
    .route(
      "/power/transitions/{id}",
      get(handlers::get_power_transition),
    )
    // BOS session from template
    .route(
      "/templates/{name}/sessions",
      post(handlers::post_template_session),
    )
    // CFS session logs (SSE)
    .route("/sessions/{name}/logs", get(handlers::get_session_logs))
    // SAT file apply — per-element endpoints. The CLI's `build_plan`
    // walks the SAT file and dispatches one POST per artifact.
    .route(
      "/sat-file/configurations",
      post(handlers::post_sat_configuration),
    )
    .route("/sat-file/images", post(handlers::post_sat_image))
    .route(
      "/sat-file/session-templates",
      post(handlers::post_sat_session_template),
    )
    // Health check
    .route("/health", get(handlers::health))
    // Hardware cluster member management
    .route(
      "/hardware-clusters/{target}/members",
      post(handlers::add_hw_component).delete(handlers::delete_hw_component),
    )
    // Hardware cluster configuration (pin/unpin)
    .route(
      "/hardware-clusters/{target}/configuration",
      post(handlers::apply_hw_configuration),
    )
    .merge(build_ws_routes())
    // Apply the global request timeout to every route in the api
    // sub-router.
    .layer(TimeoutLayer::with_status_code(
      StatusCode::REQUEST_TIMEOUT,
      state.request_timeout,
    ));

  // /api/v1/auth/* — credential-handling sub-router. No Bearer
  // extractor (chicken-and-egg). Two layered defences applied:
  // (1) per-IP rate limit, (2) body redaction from any log span.
  let limiter = AuthRateLimiter::new();
  let auth = Router::new()
    .route("/token", post(handlers::auth_token))
    .route("/validate", post(handlers::auth_validate))
    .layer(middleware::from_fn(strip_body_for_logs))
    .layer(middleware::from_fn_with_state(state.clone(), rate_limit))
    .layer(Extension(limiter));

  Router::new()
    .nest("/api/v1", api)
    .nest("/api/v1/auth", auth)
    .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
    .with_state(state)
}

/// WebSocket upgrade routes — kept separate so they're easy to identify
/// and so the upgrade protocol is not mixed with plain HTTP routes.
fn build_ws_routes() -> Router<Arc<ServerState>> {
  Router::new()
    .route("/nodes/{xname}/console", get(handlers::console_node_ws))
    .route(
      "/sessions/{name}/console",
      get(handlers::console_session_ws),
    )
}
