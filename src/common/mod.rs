/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod app_context;
pub mod audit;
pub mod authentication;
pub mod authorization;
pub mod boot_parameters;
pub mod bos_sessiontemplate_utils;
pub mod cfs_configuration_utils;
pub mod cfs_session_utils;
pub mod check_network_connectivity;
pub mod config;
pub mod hooks;
pub mod hw_inventory_utils;
pub mod ims_ops;
pub mod jwt_ops;
pub mod kafka;
pub mod kernel_parameters_ops;
pub mod local_git_repo;
pub mod log_ops;
pub mod node_ops;
pub mod pcs_utils;
pub mod user_interaction;
pub mod vault;
