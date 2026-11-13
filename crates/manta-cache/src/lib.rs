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
//! out **one call per site** — `GET /v2/groups`, every group the token
//! can access with its members — and folds the results into an
//! [`Index`]. Each call carries a new bearer token from the caller's
//! [`TokenSource`].
//!
//! A process that already holds the group data (an embedding
//! `manta-server`, a fixture-driven test) can skip HTTP entirely and
//! build the index with [`Index::from_snapshots`].
//!
//! # Example
//!
//! ```no_run
//! # async fn run() -> Result<(), manta_cache::CacheError> {
//! use manta_cache::{SiteDescriptor, TokenError, TokenSource, refresh};
//!
//! /// Gets a new token per request, e.g. with a service account's
//! /// client credentials grant.
//! struct ServiceAccount;
//!
//! impl TokenSource for ServiceAccount {
//!   async fn token(&self) -> Result<String, TokenError> {
//!     # unimplemented!()
//!   }
//! }
//!
//! let sites = [SiteDescriptor {
//!   name: "alps".into(),
//!   manta_server_url: "https://manta-server.example.ch:8443".into(),
//! }];
//!
//! let outcome = refresh(&sites, &ServiceAccount).await?;
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
mod token;

pub use error::CacheError;
pub use index::{Index, NodeMembership, SiteSnapshot};
pub use refresh::{RefreshOutcome, SnapshotOutcome, fetch_snapshots, refresh};
pub use site::SiteDescriptor;
pub use token::{TokenError, TokenSource};
