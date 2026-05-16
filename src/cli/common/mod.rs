//! Helpers used only by the CLI binary.
//!
//! Modules here have no server-side consumers and will move into the
//! `manta-cli` crate when the workspace split lands.

pub mod authentication;
pub mod display;
pub mod hooks;
pub mod kernel_parameters_ops;
pub mod local_git_repo;
pub mod user_interaction;
