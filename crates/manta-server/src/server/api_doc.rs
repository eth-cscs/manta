//! OpenAPI specification document for the manta HTTP server.

use utoipa::{
  Modify, OpenApi,
  openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use super::handlers;

/// Root OpenAPI document for the manta API.
#[derive(OpenApi)]
#[openapi(
  paths(
    handlers::health,
    handlers::get_sessions,
    handlers::get_configurations,
    handlers::get_nodes,
    handlers::get_groups,
    handlers::get_images,
    handlers::get_templates,
    handlers::get_boot_parameters,
    handlers::get_kernel_parameters,
    handlers::get_redfish_endpoints,
    handlers::get_groups_nodes,
    handlers::get_groups_hardware,
    handlers::get_clusters_deprecated,
    handlers::get_hardware_clusters_deprecated,
    handlers::get_hardware_nodes_list,
    handlers::delete_node,
    handlers::add_node,
    handlers::delete_group,
    handlers::create_group,
    handlers::add_nodes_to_group,
    handlers::delete_group_members,
    handlers::delete_boot_parameters,
    handlers::add_boot_parameters,
    handlers::update_boot_parameters,
    handlers::delete_redfish_endpoint,
    handlers::add_redfish_endpoint,
    handlers::update_redfish_endpoint,
    handlers::delete_session,
    handlers::delete_images,
    handlers::delete_configurations,
    handlers::create_session,
    handlers::apply_boot_config,
    handlers::apply_kernel_parameters,
    handlers::add_kernel_parameters,
    handlers::delete_kernel_parameters,
    handlers::migrate_nodes,
    handlers::migrate_backup,
    handlers::migrate_restore,
    handlers::create_ephemeral_env,
    handlers::post_power,
    handlers::get_power_transition,
    handlers::post_template_session,
    handlers::get_session_logs,
    handlers::post_sat_configuration,
    handlers::post_sat_image_cfs_session,
    handlers::post_sat_image_stamp,
    handlers::post_sat_session_template,
    handlers::add_hw_component,
    handlers::delete_hw_component,
    handlers::apply_hw_configuration,
    handlers::console_node_ws,
    handlers::console_session_ws,
    handlers::auth_token,
    handlers::auth_validate,
    handlers::get_available_groups,
  ),
  components(schemas(
    handlers::ErrorResponse,
    handlers::AddNodeRequest,
    handlers::AddNodesToGroupRequest,
    handlers::AddNodesToGroupResponse,
    handlers::DeleteBootParametersRequest,
    handlers::CreateSessionRequest,
    handlers::ApplyBootConfigRequest,
    handlers::KernelParamOp,
    handlers::ApplyKernelParametersRequest,
    handlers::MigrateNodesRequest,
    handlers::MigrateBackupRequest,
    handlers::MigrateRestoreRequest,
    handlers::CreateEphemeralEnvRequest,
    handlers::DeleteGroupMembersRequest,
    handlers::PowerAction,
    handlers::PowerTargetType,
    handlers::PowerRequest,
    handlers::BosOperation,
    handlers::PostTemplateSessionRequest,
    handlers::AddKernelParametersRequest,
    handlers::DeleteKernelParametersRequest,
    handlers::AddHwComponentRequest,
    handlers::DeleteHwComponentRequest,
    handlers::HwClusterMode,
    handlers::ApplyHwConfigurationRequest,
    manta_shared::types::auth::AuthTokenRequest,
    manta_shared::types::auth::AuthTokenResponse,
    manta_shared::types::auth::ValidateTokenRequest,
    crate::service::boot_parameters::UpdateBootParametersParams,
    manta_shared::types::params::redfish_endpoints::UpdateRedfishEndpointParams,
    manta_backend_dispatcher::types::Group,
    manta_backend_dispatcher::types::Member,
    manta_backend_dispatcher::types::XnameId,
    manta_backend_dispatcher::types::bss::BootParameters,
  )),
  modifiers(&SecurityAddon),
  info(
    title = "Manta API",
    version = env!("CARGO_PKG_VERSION"),
    description = "REST API for managing CSM/OpenCHAMI HPC clusters via manta.",
  )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "bearerAuth",
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
        ),
      );
    }
    // Declare the /api/v1 base path as the server so that Swagger UI
    // constructs full URLs like /api/v1/sessions when trying out calls.
    openapi.servers = Some(vec![
      utoipa::openapi::ServerBuilder::new()
        .url("/api/v1")
        .description(Some("manta API v1"))
        .build(),
    ]);
  }
}
