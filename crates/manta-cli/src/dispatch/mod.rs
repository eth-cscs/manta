//! Per-subcommand handler modules wired by `clap` to the top-level CLI
//! verbs.
//!
//! The dispatch tree mirrors the user-facing CLI: each verb
//! (`add`, `apply`, `delete`, `power`, `console`, `backup`, `restore`,
//! `migrate`, `run`, `get`, `config`, `log`, `upgrade`, `gen-man`,
//! `gen-autocomplete`) has its own module, and each leaf subcommand
//! lives in a sibling file. Every leaf `exec()` is a thin CLI adapter:
//! validate clap matches, build an OpenAPI request, dispatch via
//! [`crate::http_client::MantaClient`], and hand the response to
//! [`crate::output::action_result`] for printing.
//!
//! The CLI uses `anyhow::Error` for all handlers (see the project
//! `CLAUDE.md` for the two-tier error convention). The root dispatcher
//! lives in [`process::process_cli`].

pub mod add;
pub mod apply;
pub mod backup;
pub mod config;
pub mod console;
pub mod delete;
pub mod gen_autocomplete;
pub mod gen_man;
pub mod get;
pub mod log;
pub mod migrate;
pub mod power;
pub mod process;
pub mod restore;
pub mod run;
pub mod upgrade;
