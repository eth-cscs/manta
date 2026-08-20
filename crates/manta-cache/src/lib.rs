//! Site-resolution cache for manta.
//!
//! manta routes every command to a *site* — one CSM or OpenCHAMI
//! cluster. Normally the operator must name that site (`cli.toml`'s
//! `site = …` or `--site`). This crate removes the need for commands
//! that already name an HSM **group** or a list of node **xnames**: it
//! learns which site each group and node lives at and answers the two
//! routing lookups
//!
//! - [`Index::group_to_site`] — `group label → site`
//! - [`Index::xname_to_site`] — `xname → site`
//!
//! It caches **routing only** — never per-node state (power, boot
//! params, CFS, IMS). The canonical group membership stays in CSM /
//! OpenCHAMI.
//!
//! # How it learns the mapping
//!
//! The cache is an HTTP client of `manta-server`; it has no
//! compile-time dependency on the server's internals. [`refresh`] fans
//! out **two calls per site** — `GET /v2/groups/available` and
//! `GET /v2/groups/nodes` — and folds the results into an [`Index`].
//!
//! A process that already holds the group data (an embedding
//! `manta-server`, a fixture-driven test) can skip HTTP entirely and
//! build the index with [`Index::from_snapshots`].
//!
//! # Example
//!
//! ```no_run
//! # async fn run() -> Result<(), manta_cache::CacheError> {
//! use manta_cache::{SiteDescriptor, refresh};
//!
//! let sites = [SiteDescriptor {
//!   name: "alps".into(),
//!   manta_server_url: "https://manta-server.example.ch:8443".into(),
//!   token: "ey…".into(),
//! }];
//!
//! let outcome = refresh(&sites).await?;
//! assert!(outcome.is_complete(), "some sites failed: {:?}", outcome.failures);
//! assert_eq!(outcome.index.group_to_site("gpu-cluster"), Some("alps"));
//! # Ok(())
//! # }
//! ```
//!
//! # Scope
//!
//! This is the ROADMAP **Stage 1** deliverable: the in-process core
//! library. The HTTP API wrapper (Stage 3) and `manta-server`
//! integration / management endpoints (Stage 4) are not part of it.

// Public library API — keep it documented (docs.rs is the interface).
#![deny(missing_docs)]

mod error;
mod index;
mod refresh;
mod site;

pub use error::CacheError;
pub use index::{Index, NodeMembership, SiteSnapshot};
pub use refresh::{RefreshOutcome, SnapshotOutcome, fetch_snapshots, refresh};
pub use site::SiteDescriptor;
