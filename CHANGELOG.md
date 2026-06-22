# Changelog

All notable changes to this project will be documented in this file.

## [2.0.0-beta.59] - 2026-06-22

### Documentation

- Fix broken intra-doc links failing `cargo doc`

### Styling

- Apply rustfmt [skip ci]

## [2.0.0-beta.57] - 2026-06-22

### Bug Fixes

- Omit --dry-run hint for read-only errors on verbs without the flag

### Documentation

- Cover /sat-file/validate, create_bos_session, and add Groups guide section
- Scaffold planning README + ROADMAP for site-resolution cache
- Sweep user docs to current beta state
- Add integration test plan + prealps mock fixture
- Document `--dry-run` across all mutating verbs + add-group `-D` swap
- Add inline `--dry-run` examples to GUIDE §4/§7 + README pointer
- Design spec for hw_cluster server-to-CLI migration
- Refresh `add group --dry-run` row + `manta-server --help` snippet
- Refresh doc comments after the dry-run helper extraction
- Drop Rust helper name from GUIDE §12 dry-run paragraph

### Features

- Add common::read_only policy module
- Add CliConfiguration.read_only + AppContext.read_only fields
- Add `config set/unset read-only` toggle commands
- Gate process_cli on the read_only config setting
- Show `Read-only` line in `manta config show`
- Wire \`manta delete configurations --dry-run\` through to server
- [**breaking**] Add `--dry-run` to `manta add node`; `--disabled` short alias swaps `-d` -> `-D`
- [**breaking**] Add `--dry-run` to `manta add redfish-endpoints`; `--domain` short alias swaps `-d` -> `-D`
- [**breaking**] Add `--dry-run` to `manta apply redfish-endpoints`; `--domain` short alias swaps `-d` -> `-D`
- Add `--dry-run` to `manta delete node`
- Add `--dry-run` to `manta delete boot-parameters`
- Add `--dry-run` to `manta delete redfish-endpoints`
- Add `--dry-run` to `manta apply ephemeral-environment`
- Honor `-o` on `apply ephemeral-environment` dry-run
- Add `--dry-run` to all `manta power` mutating leaves
- Add `--dry-run` to `manta run session`
- Add `--dry-run` to `manta restore vcluster`
- [**breaking**] Add `-d` short alias to `manta add group --dry-run`

### Miscellaneous Tasks

- Remove VERBS_WITH_DRY_RUN partial-coverage allowlist

### Refactor

- Route read-only toggle messages through action_result + alphabetise subcommands
- Remove dead ArgMatchesExt import from add redfish-endpoints handler
- Consolidate dry-run helpers + dispatcher cleanups
- Drop clap dep, hand-roll minimal arg parser

### Testing

- Move delete-configurations dry-run parse tests to the binary's unit-test target
- Assert `-D` short alias for `add node --disabled` parses
- Catch up to current ServerState + get_configurations shape

## [2.0.0-beta.56] - 2026-06-16

### Build

- Regenerate openapi.json for /sat-file/validate

### Documentation

- Note hardware section gap in /sat-file/validate
- Finish --reboot -> --create-bos-session rename in GUIDE.md

### Features

- Show safe_to_delete on `manta get configurations`
- Add --only-safe-to-delete / --only-unsafe-to-delete to get configurations
- Show safe_to_delete on `manta get images`
- Add --only-safe-to-delete / --only-unsafe-to-delete to get images
- Name the timeout hop in user-facing error messages
- Add PostSatValidateRequest wire type
- Extract_all_target_groups across whole sat file
- Dispatcher impl of SatTrait::validate_sat_file
- Add post_sat_validate handler
- Wire POST /sat-file/validate route + utoipa entry
- Pre-flight SAT validation in apply sat-file
- Rename `--reboot` to `--create-bos-session` on `manta apply sat-file`
- Return a mock BOS session on dry-run + --create-bos-session

### Miscellaneous Tasks

- Trace which branch of the BOS-session match arm fired
- Bump manta-backend-dispatcher, csm-rs and ochami-rs

### Refactor

- Consolidate `manta get configurations` to a single HTTP call
- Remove `manta get analysis` and inline components-only verdict on /configurations

### Testing

- Wire-format lock for PostSatValidateRequest

## [2.0.0-beta.55] - 2026-06-14

### Bug Fixes

- Sort cache rows by image_created ascending (oldest first)
- Bump default request_timeout_secs from 300 to 600
- Sequence get_configuration_analysis upstream calls

### Documentation

- Document GET /api/v1/summary endpoint
- Surface request_timeout_secs in cli.toml samples
- Show timeout defaults as live values, not commented hints
- Add `examples/cli.toml` and `examples/server.toml`

### Features

- Add BackendSummary wire type for `/api/v1/summary`
- Pure `build_summary` linker for /summary endpoint
- Get_summary fans four service fetchers out via try_join
- Expose `GET /api/v1/summary` (Vec<BackendSummary>)
- Add `manta get summary` command
- Default one-shot REST timeout to 5 minutes
- Make every hardcoded timeout in manta-cli configurable
- Make every hardcoded timeout in manta-server configurable
- Add image_created, configuration_last_updated, session_start_time
- Rename `manta get summary` to `manta get cache`
- Print row count after `manta get cache` table
- Rename `manta get cache` to `manta get analysis image`
- Sort cache rows by image_created descending
- Add `manta get analysis configuration`
- Add --only-safe-to-delete / --only-unsafe-to-delete to analysis configuration
- Add safe_to_delete to image analysis rows
- Add --only-safe-to-delete / --only-unsafe-to-delete to analysis image

### Refactor

- [**breaking**] Drop dead `parent_hsm_group` config field and its CLI surface
- Rename summary module to cache
- Drop configuration_last_updated and session_start_time
- Complete the cache -> analysis rename
- Drop session + bos-template columns from image rows

## [2.0.0-beta.54] - 2026-06-14

### Bug Fixes

- Relax root_ca_cert_file check under --allow-http
- Collapse nested if in build.rs downconvert
- Wire missing SatTrait methods through the static dispatcher
- Cover all remaining trait methods in static dispatcher

### Build

- Generate API client from server OpenAPI spec
- Trim unused OpenAPI schemas; drop dead_code allow

### Documentation

- Qualify intra-doc link to validate_user_group_vec_access
- Qualify intra-doc link to http_client::PowerAction
- Refresh user docs for the beta.51 batch of changes
- Surface --allow-http as the test-environment shortcut in TL;DR
- Clarify auto-generated vs hand-rolled http_client split
- Explain reqwest 0.12/0.13 split in Cargo.toml
- Document `get group-hardware` → `get hardware group` rename
- Stamp 2.0.0 version headers; document --site global, glob image filter, 300s timeout

### Features

- Tighten utoipa response types for OpenAPI codegen
- Implement --pattern filter using glob syntax
- Make --site global so it's accepted after any subcommand
- [**breaking**] Move `get group-hardware` under `get hardware group`
- Add --dry-run to `add group` and `delete group`
- Unwrap progenitor ErrorResponse to `HTTP <status>: <message>`
- Propagate backend HTTP status; bump request_timeout to 300s

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]
- Regenerate shell completion scripts after get-group-hardware rename
- Regenerate man page after get-group-hardware rename
- Update .gitignore

### Refactor

- Delete service/infra_backend/ wrapper layer
- Migrate http_client to progenitor-generated client
- Collapse wire+params namespaces into api/
- Adopt ConsoleTrait resize channel + ShastaClient
- Collapse handlers/ into dispatch/
- Fix intra-doc links and document types::api modules

### Styling

- Apply rustfmt [skip ci]

### Testing

- CI guardrail that StaticBackendDispatcher covers all trait methods

## [2.0.0-beta.53] - 2026-06-09

### Miscellaneous Tasks

- Bump ochami-rs, csm-rs and manta-backend-dispatcher

## [2.0.0-beta.52] - 2026-06-09

### Build

- Bump sibling deps to mbd 1.0.0-beta.10, csm-rs 1.0.0-beta.12, ochami-rs 1.0.0-beta.9

## [2.0.0-beta.51] - 2026-06-09

### Bug Fixes

- Unify drift-bearing wire types into manta-shared::types::wire
- Unify remaining request bodies into manta-shared::types::wire
- Unify GET / DELETE query structs into manta-shared::types::wire
- Resolve group → xnames before sending /boot-config
- Move HW inventory read off the Tokio reactor in add_node
- Add `final_members` as canonical replacement for AddNodesToGroupResponse.removed
- Switch source-chain-losing .map_err(|e| anyhow!(...)) to .context()
- Drop --assume-yes / --do-not-reboot from clap where they were no-ops; quiet per-iter info logs

### Build

- Point Cargo.toml comments at the .cargo/config.toml override

### Documentation

- Describe CLI-driven 3-step image build flow
- Refresh module-level docstrings for split image flow
- Field-level rustdoc on CreateImageCfsSessionRequest
- Note the image-build sub-pipeline in CLI.md + ARCHITECTURE.md
- Refresh rustdoc, user docs, and tests after `hsm_group` rename
- Sync ARCHITECTURE + API with recent changes
- Fix six API.md drifts surfaced by the multi-agent audit
- Refresh 3 stale intra-doc links after the resolver rename
- Enable #![warn(missing_docs)] on manta-cli + document PowerAction::wire
- Clarify that to_backend exhaustiveness is rust-enforced

### Features

- CLI-driven per-image build pipeline

### Miscellaneous Tasks

- Drop unused http_client re-exports
- Confine migrate-backup/restore paths to migrate_backup_root
- Refuse plain HTTP by default + add HSTS to every response

### Performance

- Dedupe + parallelise per-image fetches in boot-config / kernel-params
- Share a process-wide reqwest::Client across Vault calls
- Swap O(N·M) Vec::contains for HashSet at cluster scale
- Loosen InfraContext wrappers to take &[String] for group-membership calls

### Refactor

- [**breaking**] Audit fixes for the per-image build pipeline
- Handler-boundary access checks + status-aware errors
- Switch remaining handlers to to_handler_error; drop dead helpers
- Unwrap server error body + tag image failures with name
- Enforce InfraContext boundary uniformly
- Wire request_timeout_secs through MantaClient::from_app_ctx
- Lift run_hook_if_present into common::hooks
- Replace .to_string() with .clone() on already-String values
- Drop async from CPU-bound service helpers
- Convert match { Some => ..., _ => ... } to if let else
- Make `()` matches explicit where the type really is `()`
- Make integer casts explicit about saturation vs domain assumption
- Tighten parameter ownership where the body doesn't need it
- Add trailing `;` to block-expression last statements
- Finish group rename + authorization-helper migration
- Split backend_dispatcher and InfraContext into per-domain files
- Rename clap arg id CLUSTER_NAME -> GROUP_NAME
- Rename + tighten authz on host-expression callers
- Drop csm-rs bypass in service/{cluster,node}
- Drop misleading `let _ = expr?;` pattern
- Collapse three service-layer too_many_arguments allows into typed params

### Testing

- Cover new image endpoints + fix stale legacy assertions
- Guard the GROUP_NAME placeholder rename
- Lock down the pure validate_group_vec_access logic

### Security

- Phase-1 authz gaps + wire-field realignment

## [2.0.0-beta.50] - 2026-06-07

### Build

- Bump sibling deps to mbd 1.0.0-beta.9, csm-rs 1.0.0-beta.11; drop [patch.crates-io]

### Features

- Show configuration, base, and groups columns in image table

## [2.0.0-beta.49] - 2026-06-06

### Bug Fixes

- Move [patch.crates-io] out of [workspace.dependencies]

### Build

- Enable csm-rs commands-admin feature and patch siblings locally
- Enable commands-admin feature on patched csm-rs path

### Miscellaneous Tasks

- Bump manta-backend-dispatch

## [2.0.0-beta.48] - 2026-06-06

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]

### Refactor

- [**breaking**] Remove legacy whole-file POST /sat-file endpoint
- [**breaking**] Drop backend methods removed in csm-rs 1.0.0-beta.8

## [2.0.0-beta.46] - 2026-06-06

### Bug Fixes

- Pre-compute GHCR image tags in shell, not via interpolation

### Styling

- Apply rustfmt [skip ci]

## [2.0.0-beta.44] - 2026-06-06

### Documentation

- Document auto-stamped manta.image_session.* metadata
- Note callers must PATCH to persist

### Features

- Stamp manta.image_session.* on freshly-built IMS images
- Fail fast with a meaningful error when manta server is down

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]
- Bump manta dependencies

### Refactor

- [**breaking**] Drop ImageSessionMetadata wrapper and read()
- [**breaking**] Move image_session metadata stamp out of manta-server

### Testing

- Cover image_session patch construction after apply

## [2.0.0-beta.43] - 2026-06-05

### Bug Fixes

- Use multi-line block form for docker tags input

## [2.0.0-beta.42] - 2026-06-05

### Bug Fixes

- Hardcode :main tag in docker push, drop metadata-action

## [2.0.0-beta.41] - 2026-06-05

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]
- Publish container images to GHCR

### Styling

- Apply rustfmt [skip ci]

## [2.0.0-beta.39] - 2026-06-05

### Features

- Add image_session module for CFS-derived IMS image metadata

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]
- Regenerate tag glob to match cargo-dist's expectation

## [2.0.0-beta.37] - 2026-06-05

### Documentation

- Disambiguate intra-doc link to print() to fix rustdoc CI

## [2.0.0-beta.36] - 2026-06-05

### Bug Fixes

- Set verify = false on manta-cli to unblock batch publish
- Remove verify = false

### Styling

- Apply rustfmt [skip ci]

## [2.0.0-beta.34] - 2026-06-05

### Bug Fixes

- Remove dead `manta apply configuration` clap def
- Add missing dispatch/run/mod.rs from the session-move commit
- Repair the cargo-release config so manta-shared still publishes

### Documentation

- Fix three stale references after the recent removals + rename
- Fix four stale Rust source docstrings
- Fix three stale apply-session references after the run/ move
- Surface the new self-care commands in user-facing docs
- Lift `run session` into its own `## run` section + fix two misnomers
- Fix run-session required-flag table (one of --group/--ansible-limit)
- Clarify which v1→v2 renames still have a grace period
- Refresh stale module/function doc comments
- Sync handler-doc subcommand lists with current clap tree
- Flip backwards "DEPRECATED alias for" doc lines
- Consolidate the 84 per-subcommand man pages into a single manta.1
- Drop references to nonexistent example config files
- Add zsh fpath note to §1.6 completion-regen section

### Features

- Gen-autocomplete now installs by default

### Miscellaneous Tasks

- Pin cargo-audit version + fix stale path in anyhow-boundary check
- Collapse the per-crate releases into one unified workspace release

### Refactor

- Move dispatch/apply/session/ to dispatch/run/session/

### Styling

- Apply rustfmt [skip ci]

### Testing

- Cover four high-value gaps in the test surface
- Cover the remaining four output renderers
- Close route-registration coverage gaps
- Broaden to_handler_error 500-fallback coverage
- Cover format_with_causes error-chain walker
- Make action_result JSON-shape tests actually test the renderer

## [2.0.0-beta.32] - 2026-06-04

### Bug Fixes

- Manta upgrade prints readable text in default mode, not JSON

### Miscellaneous Tasks

- Auto-apply rustfmt on push to main, verify on PRs

### Refactor

- Rename commands → dispatch, move process.rs into handlers

## [2.0.0-beta.30] - 2026-06-04

### Documentation

- Refresh CLI/MIGRATING/ARCH + regen man pages and completions

### Features

- [**breaking**] Surface refactor — upgrade, gen-autocomplete top-level, drop legacy aliases
- [**breaking**] End-of-session sweep — gen-man, apply-redfish/boot moves, drop deprecated v1→v2 aliases

### Miscellaneous Tasks

- Regenerate man pages + shell completions [skip ci]
- Regenerate man pages and shell completions

## [2.0.0-beta.27] - 2026-06-04

### Miscellaneous Tasks

- Refresh manta.1 with current crate version
- Auto-regenerate man pages and shell completions on push to main

## [2.0.0-beta.26] - 2026-06-04

### Documentation

- Refresh ARCHITECTURE and CLAUDE after the naming-clarity work
- Refresh three stale in-source path references

### Miscellaneous Tasks

- Drop unused `config` crate dependency

## [2.0.0-beta.25] - 2026-06-04

### Documentation

- Fix broken intra-doc link in config_summary

### Styling

- Apply rustfmt across the workspace

## [2.0.0-beta.24] - 2026-06-04

### Miscellaneous Tasks

- Clarify command help and regenerate man pages + completions
- Update CHANGELOG

### Refactor

- Tidy crates/manta-cli file structure
- Typed request structs for the 8 multi-arg http_client methods (#7a)
- Exec param structs for kernel-params + power commands (#7b, batch 1)
- Exec param structs for apply_boot/template (#7b, batch 2)
- Exec param structs for add/delete hardware + add_group/node (#7b, batch 3)
- Exec param structs for delete_config, update_* (#7b, batch 4)
- Exec param structs for migrate + apply_session (#7b, final)
- Split http_client/mod.rs into client/query/wire (#6)
- Drop the cli:: prefix from internal imports (#4)
- Promote add/get/delete/apply/update to subdirectories (#3)
- Rename files to match user-facing commands
- Tier-1+2 naming-clarity follow-up to the deep audit
- Move SAT-file Jinja2 renderer next to its only caller
- Rename manta_shared::shared module to types

## [2.0.0-beta.23] - 2026-06-04

### Miscellaneous Tasks

- Update dependencies

## [2.0.0-beta.22] - 2026-06-04

### Miscellaneous Tasks

- Fix manta version
- Bump manta version

## [2.0.0-beta.21] - 2026-06-04

### Documentation

- Regenerate CHANGELOG for the architecture-cleanup batch
- Refresh ARCHITECTURE, CLAUDE, and CI for this session's refactors
- Refresh in-source Rust docstrings for this session's refactors

### Miscellaneous Tasks

- Clear remaining workspace lints
- Bump csm-rs, ochami-rs and manta-backend-dispatch

### Refactor

- Group backend calls on InfraContext; simplify get_images
- Tighten layer boundaries on InfraContext
- Move AppContext from manta-shared to manta-cli
- Delete pure-forwarder service files
- Consolidate 18 trait files into mod.rs
- Move jwt_ops into manta-server
- Split typed config schemas + audit/kafka out of shared
- Move CLI-only sat_file Jinja2 renderer to manta-cli
- Bring redfish_endpoints into the InfraContext pattern
- Rename internal manta_backend_dispatcher module to dispatcher
- Drop dead audit_file field from CliConfiguration and ServerConfiguration

## [2.0.0-beta.20] - 2026-06-02

### Miscellaneous Tasks

- Update cli help

### Refactor

- Use typed get_images trait instead of get_images_and_details
- Filter by name regex, drop bundled tuple

## [2.0.0-beta.19] - 2026-06-01

### Miscellaneous Tasks

- Bump manta-backend-dispatcher version

## [2.0.0-beta.18] - 2026-06-01

### Documentation

- Drop the docs.rs/manta-shared row from "Where to look next"
- Add ready-to-paste curl example to every endpoint section

### Miscellaneous Tasks

- Bump manta-backend-dispatcher version

### Refactor

- Align manta-server with typed HSM trait surface; typed IMS PublicKey access

## [2.0.0-beta.17] - 2026-05-31

### Bug Fixes

- Collapse nested if-let in apply_sat_file dispatch

### Documentation

- Clarify per-module audience in lib.rs and common/mod.rs
- Correct jwt_ops audience claim in lib.rs + common/mod.rs
- Refresh CHANGELOG and ARCHITECTURE post-Tier-3.2 refactors

### Features

- Graceful shutdown on SIGTERM/SIGINT

### Miscellaneous Tasks

- Regenerate man pages + shell completions; enforce in CI
- Wire cargo-audit with documented waivers for transitive CVEs
- Code-quality polish — stale FIXME, weak tests, trailing periods
- Code-quality polish — visibility + stale phase refs
- Run as non-root user; explicit STOPSIGNAL on server

### Refactor

- [**breaking**] Drop Kafka audit emission from manta-cli
- Delete dead network probe and AppContext re-export
- Drop dead deps and methods, rename vault fetch_* to get_*
- Move delete_group_members business logic to service layer
- Move sat_file Jinja2 renderer from shared/ to common/
- Move post_power xname resolution to service layer
- Relocate node_ops from server::common to service
- Relocate authorization from server::common to service
- Fold get_restricted_boot_parameters into service
- Relocate ims_ops from server::common to service
- Nest hw_inventory_utils under service::hw_cluster

### Testing

- Extract mock_get helper for integration test fixtures

## [2.0.0-beta.16] - 2026-05-30

### Bug Fixes

- Route per-element SatTrait methods through StaticBackendDispatcher

### Documentation

- Refresh SAT-file flow and add macOS config paths
- Refresh user docs, rustdoc, and pin pure helpers
- Replace example configs with CSCS canonical site definitions

### Features

- Build an ordered execution plan in `manta apply sat-file`
- Dispatch the execution plan element-by-element
- Configurable request and per-route /power timeouts
- Default listen_address to 0.0.0.0; default port based on TLS
- Move PCS-transition polling loop to the CLI

### Miscellaneous Tasks

- Ignore crate-root cli.toml / server.toml local test configs

### Testing

- Cover per-element flow and refresh module headers

### Shore

- Update .gitignore

## [2.0.0-beta.15] - 2026-05-27

### Documentation

- Make quickstart work on macOS and from a fresh clone
- Document macOS config path everywhere, not just quickstart

### Miscellaneous Tasks

- Bump csm-rs to 1.0.0-beta.2

## [2.0.0-beta.14] - 2026-05-26

### Bug Fixes

- Log full source() chain on handler errors
- Adapt to csm-rs API drift (Csm::new fallible, ims tokens per-call)

### Documentation

- Refresh API.md / ARCHITECTURE.md / GUIDE.md for recent changes
- Remove the dead [sites.*] block from the v2 cli.toml example
- Correct service-module count, SAT-section count, CLI binary name

### Features

- Show timestamps in server log output

### Miscellaneous Tasks

- Update Cargo.toml

### Refactor

- Use ShastaClient API for IMS calls
- [**breaking**] Move render + filter + preview to CLI
- Pipe parsed Value through trait; delete manta-shared SatFile

## [2.0.0-beta.13] - 2026-05-23

### Documentation

- Add Migrating section + Unreleased CHANGELOG for Tier 3.2
- Document new /groups/nodes and /groups/hardware endpoints
- Rewrite per-section commands to use canonical group-centric names
- Rewrite example commands to use canonical group-centric names
- Add MIGRATING.md (v1 -> v2 upgrade guide)
- Close out [Unreleased] CHANGELOG; fix cli.toml.example
- Refresh module headers + fn docstrings after Tier 3.2 renames

### Features

- Route apply session + apply sat-file through output::action_result
- Structured renderer for config show with --output json
- Introduce 'add nodes' / 'delete nodes' under add/delete verbs
- Promote vCluster backup/restore to top-level verbs
- Rename 'apply session' to 'run session'
- Introduce 'get group-nodes' / 'get group-hardware' (Tier 3.2 phase 1/N)
- Introduce 'apply boot group' (Tier 3.2 phase 2/N)
- Introduce 'apply hardware group' + group-flag aliases (Tier 3.2 phase 3/N)
- Introduce 'power on/off/reset group' (Tier 3.2 phase 4/N)
- Rename /clusters and /hardware-clusters REST paths (Tier 3.2 phase A6)

### Miscellaneous Tasks

- Update gitignore

### Refactor

- Rename cluster-named command modules (Tier 3.2 phase 5/N)

## [2.0.0-beta.12] - 2026-05-23

### Build

- Bump direct deps to clear cargo-audit advisories

### Documentation

- Add Troubleshooting section with curl recipes
- Summarize the 22 commits since 2.0.0-beta.11

### Features

- Expand client-authentication tracing
- Trace backend auth calls in service layer
- Trace dispatcher boundary into csm-rs/ochami-rs
- Log the path the server configuration was loaded from
- Log a startup summary of the effective configuration
- Log every HTTP request as a copy-pasteable curl command
- Route add_* commands through output::action_result
- Route delete_* commands through output::action_result
- Route update/apply/migrate commands through output::action_result
- Route config/power commands through output::action_result

### Miscellaneous Tasks

- Build both Dockerfiles in CI to catch image regressions in PR

### Refactor

- Replace 2-line handler preamble with `ctx.infra()`
- Use f64 + scoped allow for the precision-loss casts
- Silence struct_excessive_bools on 4 audited structs
- Print startup config summary to stdout
- Add ArgMatchesExt to dedupe arg-extraction boilerplate
- Rename --hsm-group to --group with backwards-compat alias
- Pluralize redfish-endpoint subcommand for consistency
- Flatten arbitrary command directory splits

### Styling

- Cargo clippy --fix sweep for cosmetic lints

## [2.0.0-beta.11] - 2026-05-22

### Documentation

- Document pub fields/variants across small param + dto + error modules
- Finish Phase 1 of rustdoc — sat_file + config types + 4 doctests + CI
- Finish Phase 2 of rustdoc — close 82 missing-docs sites in manta-server
- Publish workspace rustdoc to GitHub Pages on push to main

## [2.0.0-beta.10] - 2026-05-21

### Miscellaneous Tasks

- Pin Rust toolchain to 1.92.0 (local + CI in lockstep)

### Styling

- Cargo fmt baseline across 9 files touched in recent commits

## [2.0.0-beta.9] - 2026-05-20

### Bug Fixes

- Update cli_tests.rs after the manta-cli -> manta binary rename

## [2.0.0-beta.8] - 2026-05-20

### Bug Fixes

- `add node -d/--disabled` actually disables the node

### Build

- Rename binary to 'manta'
- Per-crate Dockerfiles for manta-cli + manta-server
- Bump builder image to rust:1.88-bookworm

### Documentation

- Changelog entry for the 10 unreleased test commits
- Remove four implemented plan documents
- Remove references to commands that no longer exist
- Fix port, missing endpoints, phantom endpoint, config field shape
- Update post-workspace-split references and tool names
- Drop stale claims; rebuild Dockerfile for the workspace
- Fix structural counts, module lists, and overstated claims
- Correct flag requirements + log alias + redfish-endpoints repeatability
- Fix session_type and POST /power enum values
- Tighten routes.rs endpoint count
- Add TL;DR sections to README, GUIDE, CLI, API

### Miscellaneous Tasks

- Fix cliff.toml duplication and regenerate CHANGELOG

### Refactor

- Use get_flag for `add node --disabled`

### Testing

- Cover AuthRateLimiter window-reset and pruning logic
- Compact route smoke tests into table-driven form
- Cover untested BackendError-to-HTTP variants in to_handler_error
- Cover config loader env-var override and error paths
- Cover MantaError-to-BackendError mapping in wire_conv::to_backend
- Cover QueryBuilder and ws_base_url in http_client
- Cover console-bridge inactivity timeout via ConsoleSocket trait
- Cover compute_summary_status and hardware-aggregation helpers
- Extract pure JSON builders in audit + cover wire shape
- Cover non-IO Kafka surface (new, clone, Debug)

## [2.0.0-beta.7] - 2026-05-19

### Miscellaneous Tasks

- Install cmake + libcurl4-openssl-dev before cargo build

## [2.0.0-beta.4] - 2026-05-18

### Build

- Per-crate license-file declarations to fix cargo-dist asset copy

## [2.0.0-beta.3] - 2026-05-18

### Build

- Drop sibling-repo path deps from workspace, registry-only
- Write generated man pages and completions to OUT_DIR
- Bump workspace manta-shared dep to =2.0.0-beta.2

### Miscellaneous Tasks

- Update Cargo.toml

## [2.0.0-beta.1] - 2026-05-18

### Bug Fixes

- Resolve all compiler warnings
- Stream session logs through manta server in server mode
- Normalize manta_server_url scheme and read from per-site config
- Resolve node expression to xnames in get_hardware_nodes_list
- Force argument is the oposite of gradeful

### Build

- Generate man pages and shell completions via build.rs
- Convert to Cargo workspace with crates/manta-cli as the sole member
- Extract manta-shared as a library crate
- Move backend dispatcher into manta-shared
- Move common/ into manta-shared
- Extract manta-server as a separate binary crate
- Make manta-shared publishable; add version pins to workspace deps

### Documentation

- Update CLI.md hardware output format options
- Update CLI.md for hostlist expression support
- Quote hostlist expressions in CLI examples
- Overhaul all help text for clarity and technology-agnostic language
- Sync API.md, ARCHITECTURE.md, and README.md with current code
- Document Cargo workspace split + scope CI fmt/grep paths
- Document the per-binary config files
- Close out Phase 6 (auth via server) — CHANGELOG + ARCHITECTURE
- Close out Phase 7 (CLI ↔ backend decoupling)
- Replace stale workspace-root config.toml with cli/server examples
- Sync README/ARCHITECTURE/CHANGELOG/CLAUDE/API/CLI/GUIDE/CI with current code
- Sync CHANGELOG and ARCHITECTURE for the four maintainability commits

### Features

- Complete CLI→server forwarding and optional TLS for manta serve
- Wire up hardware cluster table output
- Wire up hardware node table output
- Add redfish_endpoints output module
- Wire up redfish-endpoints table output
- Restore full hardware cluster output modes (summary/details/pattern/json)
- Add 'get hardware nodes' command
- Add OpenAPI spec and Swagger UI to the HTTP server
- Add /api/v1/auth/{token,validate} endpoints with 4 mitigations
- Add GET /groups/available + /groups/all endpoints

### Miscellaneous Tasks

- Update Cargo.toml
- Remove dead code and fix all clippy warnings in src/
- Untrack stray runtime config.toml; ignore /crates/*/config.toml
- Add LICENSE file to creates
- Update Cargo.toml

### Refactor

- Multi-site HTTP server — one server, many clusters
- Thin CLI client — route commands through manta HTTP server
- Move hw-cluster logic to service layer, fix layer inversion
- Thin CLI — all commands route through manta HTTP server
- Resolve hosts expressions server-side for all node-list commands
- Remove redundant 'get hardware node' command
- Route post_power through service::power
- Move SAT YAML types out of cli/ into common/
- Extract src/shared/ for wire-shared types (Params + DTOs)
- Partition common/ by ownership; carve out cli/common/ and server/common/
- Pull cross-layer helpers out of service/server-only modules
- Remove per-site manta_server_url field
- Add CliConfiguration and ServerConfiguration schemas
- Add per-binary loaders alongside the legacy one
- Load server.toml; flags override config
- Load cli.toml; retarget config edit subcommands
- Expand NotFound errors with sample + migration mapping
- Delete legacy MantaConfiguration + get_configuration
- Make manta_server_url required; drop the always-Some dance
- Authentication.rs uses MantaClient instead of backend
- Migrate apply_session.rs to MantaClient; server validates
- Migrate add hardware; validate in 3 hw_cluster handlers
- Migrate migrate-nodes; validate in migrate_nodes handler
- Migrate 4 config_* commands to MantaClient
- Drop StaticBackendDispatcher construction from CLI runtime
- Split hw_cluster.rs (2925 LOC) into mod tree
- Bundle State+BearerToken+SiteName into RequestCtx
- Collapse MantaClient query-building into QueryBuilder
- Split build.rs (1311 LOC) per command family
- Decouple from csm-rs / ochami-rs / manta-backend-dispatcher
- Split handlers.rs (3316 LOC) into per-resource modules
- Flatten AppContext — drop CliInfra/CliConfig wrappers
- Slim CliConfiguration and drop dead Site vault fields
- Move backend bridge from manta-shared to manta-server
- Mirror NodeDetails locally, drop csm-rs from manta-shared
- Introduce MantaError, drop BackendError from helpers
- Split http_client.rs (1254 LOC) into per-resource modules
- Move large inline test blocks to sibling files
- Move 1864 LOC of tests to crates/manta-server/tests/
- Collapse the two crate::common re-export shims

### Styling

- Cargo fmt baseline after workspace split

### Testing

- Add coverage for GET /api/v1/hardware-nodes-list
- Fix stale test, remove dead helper, add 501-complement tests

### Fox

- Cargo.toml files

## [1.64.3] - 2026-05-08

### Features

- Field 'name' in configuration layer is optional

### Miscellaneous Tasks

- Update Cargo.toml

## [1.64.0] - 2026-05-08

### Bug Fixes

- Command migrate nodes
- Force rust-ls crypto provider to 'ring'

### Miscellaneous Tasks

- Update Cargo.toml
- Update .gitignore file

## [1.63.1] - 2026-05-05

### Documentation

- Add rustdoc to service, common, and server layers
- Complete rustdoc coverage for handlers, service structs, and CLI modules

## [1.63.0] - 2026-05-03

### Bug Fixes

- Use BearerToken extractor for WebSocket console endpoints

### Documentation

- Add HTTP API reference (API.md)

### Features

- Add HTTPS server mode with 12 GET API endpoints
- Add 14 write API endpoints (CRUD + deletion with dry_run)
- Add 5 HTTP endpoints (power, template sessions, SSE logs, group member removal, SAT file)
- Add 6 remaining HTTP endpoints and code quality improvements
- Add 60-second request timeout to HTTP server
- Classify service errors into 404/409 instead of always returning 500
- Migrate log→tracing and add service-layer unit tests (#10, #1)
- Add WebSocket console endpoints for nodes and CFS sessions
- HTTP server, WebSocket console, error type cleanup, and test coverage

### Miscellaneous Tasks

- Switch to local path dependencies for development
- Ignore .claude session directory

### Refactor

- Apply code quality improvements to HTTP server layer
- Apply code quality improvements across config, service, and main
- Remove duplicate backend construction and add request logging middleware
- Normalize HTTP response status codes and shapes
- Replace if/else-if dispatch chains with match cli.subcommand()
- Phase 1 quick fixes (helpers, dedup, route grouping)
- Propagate manta_backend_dispatcher::error::Error through service layer
- Remove anyhow from common layer and add HTTP integration tests
- Enforce anyhow/BackendError boundary in CLI and add CI gate
- Replace Error::Message with typed error variants

### Testing

- Add HTTP server smoke tests (35 tests, no real backend needed)

## [1.62.9] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.8] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.7] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.6] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.5] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.4] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.3] - 2026-04-19

### Miscellaneous Tasks

- Try to fix github workflow to build artifacts

## [1.62.2] - 2026-04-19

### Miscellaneous Tasks

- Try newest version of rdkafka crate

## [1.62.1] - 2026-04-19

### Miscellaneous Tasks

- Update cargo dist

## [1.62.0] - 2026-04-19

### Bug Fixes

- Code review priorities 1-3 — bugs, robustness, and code quality

### Features

- Add --site flag for per-invocation site selection

### Miscellaneous Tasks

- Update .gitignore file
- Update Cargo.toml
- Update .gitignore file

### Refactor

- Code review priority 4 — structural refactoring
- Code review priority 5 — DRY helper extraction for audit and node resolution
- Code review priority 6 — named constants, Display formatting, path constants, and typo fix
- Migrate all 12 get commands to service layer with auth centralization
- Split AppContext into InfraContext + CliConfig
- Migrate Batch 1 write commands (10 commands) to service layer pattern
- Migrate Batch 2 kernel parameter commands to service layer pattern
- Migrate Batch 3 moderate write commands (7 commands) to service layer pattern
- Migrate Batch 4 migrate commands (3 commands) to service layer pattern
- Migrate Batch 5+6 complex commands to service layer pattern
- Centralize get_api_token in all remaining handlers (Batch 7)

### Testing

- Add 62 unit tests across 7 modules (73 -> 135 total)
- Add unit tests and refactor kernel_parameters_ops for testability

## [1.61.2] - 2026-04-06

### Miscellaneous Tasks

- Update Cargo.lock

### Refactor

- Clean unnecessary files

## [1.61.1] - 2026-03-27

### Bug Fixes

- Improve error message when command 'delete configuration' does not have enough context to find out if user has access to delete the requested configuration

### Features

- Add new argument '--most-recent' to command 'get images'
- Add new argument '--most-recent' to command 'get images'

## [1.61.0] - 2026-03-09

### Bug Fixes

- Effort 6 — bugs, security, quality improvements, and dep updates
- When processing a SAT file, images are not searched using a fuzzy finder but a name match

### Documentation

- Add doc comments to all public APIs across 77 files (P19)

### Refactor

- Comprehensive code quality improvement (Efforts 1-4, P1-P3)
- Complete Effort 4 code quality improvements (P4-P17)
- Complete Effort 5 code quality improvements (P1-P16, P2)
- Add dispatch! macro for backend dispatcher (P12) and replace termion with crossterm (P15)
- Extract send_audit() helper to deduplicate audit message construction (P17)
- Decompose top 5 largest functions into smaller helpers (P14)

### Testing

- Add unit tests for 31 pure functions across 4 modules (P18)

## [1.60.4] - 2026-02-27

### Bug Fixes

- Power management operations not able to deserialize struct response when creating a transition

### Refactor

- Update Cargo.lock

## [1.60.3] - 2026-02-24

### Bug Fixes

- Rollback rdkafka to overcome error compiling librdfafka
- Rollback rdkafka to overcome error compiling librdfafka

## [1.60.2] - 2026-02-24

### Bug Fixes

- Rollback rdkafka to overcome error compiling librdfafka

## [1.60.0] - 2026-02-24

### Bug Fixes

- Improve sat file eschema parsing

### Refactor

- Clean code related to command apply sat-file
- Update Cargo.toml

## [1.59.9-beta.16] - 2026-02-01

### Bug Fixes

- Power operations now operates on a struct instead of a serde json value

## [1.59.9-beta.15] - 2026-02-01

### Refactor

- Assume_yes user interaction code cleaned
- Assume_yes user interaction code cleaned

## [1.59.9-beta.14] - 2026-01-31

### Refactor

- Clean code

## [1.59.9-beta.13] - 2026-01-31

### Refactor

- Clean code

## [1.59.9-beta.12] - 2026-01-31

### Refactor

- Clean code

## [1.59.9-beta.11] - 2026-01-31

### Refactor

- Clean code

## [1.59.9-beta.10] - 2026-01-31

### Refactor

- Clean code

## [1.59.9-beta.9] - 2026-01-31

### Refactor

- Clean code
- Clean code

## [1.59.9-beta.8] - 2026-01-31

### Refactor

- Clean code
- Clean code

## [1.59.9-beta.7] - 2026-01-30

### Refactor

- Clean code

## [1.59.9-beta.6] - 2026-01-30

### Refactor

- Clean code

## [1.59.9-beta.5] - 2026-01-30

### Refactor

- Clean code

## [1.59.9-beta.4] - 2026-01-30

### Refactor

- Clean code

## [1.59.9-beta.3] - 2026-01-30

### Refactor

- Add cli handlers to clean code

## [1.59.9-beta.2] - 2026-01-26

### Features

- Add wiki badge

### Miscellaneous Tasks

- Clean code
- Refactor code
- Refactor code
- Refactor code

### Refactor

- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code

## [1.59.9-beta.1] - 2026-01-22

### Bug Fixes

- Command 'manta delete' not filtering the data to delete
- Command to get the list of hsm groups available not filtering the system wide hsm groups

### Miscellaneous Tasks

- Update Cargo.toml

### Refactor

- Adapt code to csm-rs clean-code branch
- Clean code
- Cargo fix
- Update .gitignore
- Clean code
- Update cargo-dist
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Clean code
- Manta no longer process host expressions

### Regactor

- Clean code

## [1.59.8] - 2026-01-09

### Bug Fixes

- Exit log streaming if git-clone init container is terminated with exit code different than 0
- Exit log streaming if git-clone init container is terminated with exit code different than 0

## [1.59.7] - 2026-01-06

### Features

- Refactor code related to configuration and data related deletion

## [1.59.6] - 2025-12-11

### Miscellaneous Tasks

- Update cli help

## [1.59.5] - 2025-12-10

### Miscellaneous Tasks

- Update cargo dist configuration and pipeline

## [1.59.4] - 2025-12-09

### Miscellaneous Tasks

- Improve error management

## [1.59.3] - 2025-12-09

### Miscellaneous Tasks

- Cargo fix
- Update Cargo.toml
- Clean Cargo.lock

## [1.59.2] - 2025-12-08

### Features

- Improve error management

## [1.59.1] - 2025-12-07

### Miscellaneous Tasks

- Update Cargo.toml
- Update .gitignore and Cargo.lock

## [1.59.0] - 2025-12-04

### Bug Fixes

- Operation apply sat file canceled if user is does not accept the rendered file file

## [1.58.10] - 2025-12-04

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.lock

## [1.58.9] - 2025-12-04

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.lock

## [1.58.8] - 2025-12-01

### Miscellaneous Tasks

- Update csm-rs crate to fix bugs related to From trail for CSF Component struct
- Update Cargo.lock

## [1.58.7] - 2025-12-01

### Miscellaneous Tasks

- Update help text related to apply sat-file

## [1.58.6] - 2025-12-01

### Bug Fixes

- Command 'get templates' crashes if a templage has 'None' in field(s) .cfs.configuration

## [1.58.5] - 2025-12-01

### Features

- Table showing results from command 'get templates' are not showing etag information anymore

## [1.58.4] - 2025-11-28

### Miscellaneous Tasks

- Update Cargo.lock

### Shore

- Update Cargo.toml

## [1.58.3] - 2025-11-28

### Bug Fixes

- Get sessions fails because it can neither filter by hsm group nor xname properly

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.lock

## [1.58.2] - 2025-11-25

### Bug Fixes

- Update Cargo.toml
- Update Cargo.toml

## [1.58.1] - 2025-11-21

### Bug Fixes

- Add domain 'https://api.cmn.alps.cscs.ch' as valid when accessing CFS configration layers urls
- Add domain 'https://api.cmn.alps.cscs.ch' as valid when accessing CFS configration layers urls

## [1.58.0] - 2025-11-13

### Features

- Improve the way hardware components needs are when debugging hardware components across groups
- Improve the way hardware components needs are when debugging hardware components across groups
- Apply sat-file now asks user for confirmation is SAT template has a session_template since it will most likely reboot the nodes

### Miscellaneous Tasks

- Improve comments and messages by replacing 'HSM' with 'group'

## [1.57.3] - 2025-11-02

### Bug Fixes

- Command apply-sat was creating wrong bos sessiontemplate by adding field 'node_groups: []' when 'node_groups' was not used in the sat template file. This patch address this and field 'node_groups' won't be added to the bos sessiontemplate submitted to CSM if field missing in SAT template file to avoid CSM from complainning and rejecting the bos sessiontemplate creation
- Command apply-sat was creating wrong bos sessiontemplate by adding field 'node_groups: []' when 'node_groups' was not used in the sat template file. This patch address this and field 'node_groups' won't be added to the bos sessiontemplate submitted to CSM if field missing in SAT template file to avoid CSM from complainning and rejecting the bos sessiontemplate creation

## [1.57.2] - 2025-11-01

### Bug Fixes

- Wrong root ca public cert for a site won't exit the application. This makes command 'manta config more resilient'

## [1.57.1] - 2025-10-29

### Bug Fixes

- Delete data

### Miscellaneous Tasks

- Update Cargo.lock

## [1.57.0] - 2025-10-24

### Features

- Delete image dryrun goes through the list of images and prints image json

## [1.56.17] - 2025-10-19

### Miscellaneous Tasks

- Bump csm-rs and ochami-rs libraries
- Update Cargo.lock

## [1.56.16] - 2025-10-19

### Miscellaneous Tasks

- Update csm-rs with fixes to improve CFS logs management
- Update Cargo.lock

## [1.56.15] - 2025-10-17

### Miscellaneous Tasks

- Update Cargo.toml
- Remove Cargo.lock file
- Remove Cargo.lock file
- Update Cargo.toml

## [1.56.9] - 2025-10-03

### Features

- Printing table with groups was breaking the line in the middle of members. This feature tries to fix this by setting ',' delimiter

## [1.56.8] - 2025-10-02

### Features

- Find sessions by type

### Miscellaneous Tasks

- Update Cargo.lock

## [1.56.7] - 2025-09-23

### Miscellaneous Tasks

- Update Cargo.toml

## [1.56.6] - 2025-09-23

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.toml

## [1.56.5] - 2025-09-23

### Miscellaneous Tasks

- Update 'cargo dist'
- Do not publish in crate.io

## [1.56.2] - 2025-09-21

### Miscellaneous Tasks

- Update Cargo.toml

### Refactor

- Cargo fix

## [1.56.1] - 2025-09-11

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.lock

## [1.55.6] - 2025-09-08

### Features

- Add a new argument to print timestamps to session logs

### Miscellaneous Tasks

- Update ochami-rs version in Cargo.toml

## [1.55.5] - 2025-09-07

### Features

- Sessions to configure nodes or build images may run slow, this patch tries to help sysadmins analyse the health of the system by showing the session completion time and duration also, it simplifies the values for completion and succeeded

## [1.55.4] - 2025-09-03

### Bug Fixes

- Manta was not validating the auth token fetched from filesystem, this patch validates the authentication token fetched from file system

### Features

- [**breaking**] Function to delete configuration and data related now accepts glob as configuration name input

### Refactor

- [**breaking**] Reduce the number of memory allocations

## [1.55.3] - 2025-08-24

### Miscellaneous Tasks

- Update Cargo.toml
- Update Cargo.toml

## [1.55.2] - 2025-08-24

### Refactor

- Update to newer version of backend dispatcher

## [1.54.1-beta.181] - 2025-08-19

### Refactor

- Migrate BOS session from json serde Value to struct

## [1.54.1-beta.180] - 2025-08-19

### Bug Fixes

- Command 'apply kernel-parameters' was not logging the right information when running --dry-run argument and it was not applying the kernel parameters properly

### Features

- Add new argument --overwrite to command 'add kernel-parameters' to is overwrites kernel parameter value is key already exists
- Add new argument --overwrite to command 'add kernel-parameters' to is overwrites kernel parameter value is key already exists

### Refactor

- Clean code

## [1.54.1-beta.179] - 2025-08-15

### Features

- Add dry-run argument to commands 'add kernel-parameters' and 'delete kernel-parameters'
- Commands related to kernel parameters improve the user prompt by using host lists to list nodes affected by user operations
- [**breaking**] Changes in command "get cluster" --status argument changes to --status-summary and --status now filters by status

### Refactor

- Apply cargo fix

## [1.54.1-beta.178] - 2025-08-14

### Features

- Change printing table as dynamic to force table width to fit in screen
- Command get boot-parameters only supports a list of xnames as an entry point, however, manta supports host expression and this patch fixes this by adding support to HSM group name and host expressions

## [1.54.1-beta.177] - 2025-08-13

### Features

- The way manta prints the table with kernel parameters never felt right because it would try to arrange values depending on a threshold calculated based on the largest kernel parameter. This patch address this by using comfy-table dynamic table to adapt table width to the screen size

## [1.54.1-beta.176] - 2025-08-13

### Features

- CSM 1.6.2 provides iSCSI which requires etag and image id in kernel parameters to match. Command 'apply boot' now makes sure etag in kernel param matches with the image id

### Refactor

- Update Cargo.lock

## [1.54.1-beta.175] - 2025-08-11

### Miscellaneous Tasks

- Update Cargo.lock

### Refactor

- We are trying to improve the quality of the code by improving its structure. This patch addresses this for the code related to delete configurations and derivatives by moving the code to its own module

## [1.54.1-beta.174] - 2025-08-03

### Features

- Move interactive code in functionality to delete and cancel CFS sessions to higher levels
- Move interactive code in functionality to delete and cancel CFS sessions to higher levels

### Miscellaneous Tasks

- Clean data

## [1.54.1-beta.173] - 2025-08-01

### Features

- Power management command now shows a summary of the nodes affected by the operation. The summary contains a hostlist to make the summary more readable

### Miscellaneous Tasks

- Clean code

## [1.54.1-beta.172] - 2025-07-30

### Features

- Add functinality dry-run to command apply kernel-parameters

### Miscellaneous Tasks

- Update Cargo.lock

## [1.54.1-beta.171] - 2025-07-29

### Bug Fixes

- BOS boot_set.rootfs_provider value was hardcoded to 'cpss3' and this is incompatible with iSCSI. This fix sets boot_set.rootfs_provider in bos to the same value user specifies in the SAT file
- BOS boot_set.rootfs_provider value was hardcoded to 'cpss3' and this is incompatible with iSCSI. This fix sets boot_set.rootfs_provider in bos to the same value user specifies in the SAT file

### Refactor

- Rename variable

## [1.54.1-beta.170] - 2025-07-29

### Bug Fixes

- We are still using cargo dist 'dirty' which means github pipeline is not checked/validated because it assumes it has been modified by the user which is true due to the discontinuation of cargo dist and the necesity to upload the gitlab vm image. This fix is to update the cargo dist version to install in the vm to the most recent one which is the one we are now using

## [1.54.1-beta.169] - 2025-07-28

### Bug Fixes

- We are still using cargo dist 'dirty' which means github pipeline is not checked/validated because it assumes it has been modified by the user which is true due to the discontinuation of cargo dist and the necesity to upload the gitlab vm image. This fix is to update the cargo dist version to install in the vm to the most recent one which is the one we are now using

## [1.54.1-beta.168] - 2025-07-28

### Miscellaneous Tasks

- Cargo dist is now back and this commit is to push newest cargo dist version and configurations

## [1.54.1-beta.167] - 2025-07-28

### Bug Fixes

- Remove interactive features in function apply_session
- Remove interactive features in function apply_session

### Miscellaneous Tasks

- Commit Cargo.lock (#99)
- Clean code
- Remove files copied by mistake feat: when deleting an image, code validates the image belongs to a node the user has access to and also check the image is not being used to boot a node
- Add Cargo.lock file
- Update Cargo.toml
- Update Cargo.toml
- Clean Cargo.toml
- Clean Cargo.toml

## [1.54.1-beta.155] - 2025-07-14

### Miscellaneous Tasks

- Github pipeline fails when building musl binary, this commit will address this by removing the musl binary from the pipeline

## [1.54.1-beta.154] - 2025-07-14

### Miscellaneous Tasks

- Github pipeline fails becuase rdkafka can't be compiled. This commit addresses this by increasing the rdkafka version

## [1.54.1-beta.153] - 2025-07-11

### Miscellaneous Tasks

- Update rust version

## [1.54.1-beta.152] - 2025-07-11

### Features

- Add new flag '--overwrite-configuration' to command apply sat-file to overwrite and clean images if a CFS configuration needs to be overwritten

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.151] - 2025-06-14

### Bug Fixes

- This patch fixes compilation errors because manta-backend-dispatcher did not have a default code in function to get power status

## [1.54.1-beta.149] - 2025-06-01

### Features

- Some cli commands had argument --dry-run while other --no-dryrun this was confusing and this patch tries to normalize all commands to use --dry-run

### Miscellaneous Tasks

- Try to build musl binary

## [1.54.1-beta.147] - 2025-05-22

### Bug Fixes

- Musl target compilation failure because host can't find openssl/libssl library. To fix this issue, we need to add vendored feature to openssl and this is what we are doing here with the  in request

## [1.54.1-beta.146] - 2025-05-22

### Miscellaneous Tasks

- Update github runner for musl artifact

## [1.54.1-beta.145] - 2025-05-22

### Miscellaneous Tasks

- Adding a musl target to cargo dist to have glibc statically compiled

## [1.54.1-beta.144] - 2025-05-22

### Miscellaneous Tasks

- Manta fails in bastion-alps because the version of GLIBC is too old there, the glibc version in the github runner is 2.39 but the one in bastion-alps is 2.38. This patch will set the github runner version to ubuntu-22.04 instead of ubuntu-latest

## [1.54.1-beta.143] - 2025-05-21

### Bug Fixes

- Command 'get images' fail because the date format in json image.created field does not have he same format as before. This patch fixes this so manta will formant the date with or without timezone

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.140] - 2025-05-12

### Bug Fixes

- Get templates

### Refactor

- Apply new format rules
- Format code with new rules

## [1.54.1-beta.139] - 2025-05-09

### Bug Fixes

- Add boot parameters

## [1.54.1-beta.138] - 2025-05-09

### Bug Fixes

- Command to add nodes to a group

## [1.54.1-beta.137] - 2025-05-09

### Features

- Add argument '--do-not-reboot' to subcommands 'add kernel-parameters', 'apply kernel-parameters' and 'delete kernel-parameters'

### Refactor

- Clean code

## [1.54.1-beta.136] - 2025-05-05

### Bug Fixes

- Get hsm member summary

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.135] - 2025-05-05

### Bug Fixes

- Manta audit breaks if JWT token does not have fields name and user_id

## [1.54.1-beta.134] - 2025-05-03

### Bug Fixes

- Errors when running subcommand 'get redfish-endpoints'

## [1.54.1-beta.133] - 2025-05-03

### Miscellaneous Tasks

- Udpate Cargo.toml

### Refactor

- Clean code

## [1.54.1-beta.132] - 2025-04-26

### Refactor

- Clean code

## [1.54.1-beta.131] - 2025-04-26

### Features

- Send terminal size to backend when connecting to node console

## [1.54.1-beta.130] - 2025-04-26

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.129] - 2025-04-23

### Refactor

- Clean code

## [1.54.1-beta.128] - 2025-04-23

### Bug Fixes

- Github workload

## [1.54.1-beta.127] - 2025-04-22

### Bug Fixes

- Github pipeline

## [1.54.1-beta.126] - 2025-04-22

### Bug Fixes

- Github pipeline

## [1.54.1-beta.125] - 2025-04-22

### Miscellaneous Tasks

- Update cargo dist configuration
- Update ubuntu image version (github runner) in github pipeline

## [1.54.1-beta.124] - 2025-04-21

### Bug Fixes

- Delete and cancel session

### Features

- Update cargo dist configuration to update github runner to ubuntu-22.04

### Miscellaneous Tasks

- Cargo fix
- Clean code
- Cargo fix
- Clean code
- Clean code
- Update Cargo.toml

### Refactor

- Merge traits related to BOS
- Clean code
- Clean code

### Shore

- Clean code

## [1.54.1-beta.122] - 2025-04-18

### Bug Fixes

- Sat file schema compatibility

## [1.54.1-beta.121] - 2025-04-18

### Bug Fixes

- Variable name
- Rollback to CFS v2

### Refactor

- Organize function to filter cfs configurations

## [1.54.1-beta.120] - 2025-04-15

### Features

- Remove 'mesa' dependencies in subcommand 'log'

## [1.54.1-beta.119] - 2025-04-15

### Features

- Remove 'mesa' dependencies in subcommand 'log'

## [1.54.1-beta.118] - 2025-04-15

### Features

- Remove 'mesa' dependencies in subcommand 'apply sat-file'

## [1.54.1-beta.117] - 2025-04-15

### Miscellaneous Tasks

- Clean code and update backend dispatcher trait function

## [1.54.1-beta.116] - 2025-04-15

### Bug Fixes

- Import of backend dispatcher CFS files

## [1.54.1-beta.115] - 2025-04-14

### Features

- Remove 'mesa' dependencies in file 'migrate_nodes_between_hsm_groups'

## [1.54.1-beta.114] - 2025-04-14

### Features

- Remove 'mesa' dependencies in subcommand 'migrate restore'

## [1.54.1-beta.113] - 2025-04-14

### Features

- Remove 'mesa' dependencies in file 'remove_nodes_from_hsm_groups.rs'

## [1.54.1-beta.112] - 2025-04-14

### Features

- Remove 'mesa' dependencies in subcommand 'delete images'

## [1.54.1-beta.111] - 2025-04-14

### Features

- Remove 'mesa' dependencies in subcommand 'apply templates'

### Miscellaneous Tasks

- Clean code

## [1.54.1-beta.110] - 2025-04-13

### Refactor

- Clean code

## [1.54.1-beta.109] - 2025-04-13

### Features

- Remove 'mesa' dependencies in subcommand 'add nodes to hsm groups'

## [1.54.1-beta.108] - 2025-04-13

### Features

- Remove 'mesa' dependencies in subcommand 'get templates'

## [1.54.1-beta.106] - 2025-04-12

### Features

- Add flag '--do-not-reboot' to subcommand 'apply boot'

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.105] - 2025-04-10

### Miscellaneous Tasks

- Update ochami-rs version

## [1.54.1-beta.104] - 2025-04-09

### Features

- Add member also includes member into group

## [1.54.1-beta.103] - 2025-04-09

### Miscellaneous Tasks

- Subcommand 'add node' no longer requires hardware inventory file

## [1.54.1-beta.102] - 2025-04-03

### Bug Fixes

- Remove argument '--nodes' in command 'power off nodes'

## [1.54.1-beta.101] - 2025-04-02

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.100] - 2025-03-31

### Bug Fixes

- Normalize command to get/add/delete/apply kernel parameters

## [1.54.1-beta.99] - 2025-03-27

### Features

- Add new command 'apply kernel-parameters'
- Add new command 'apply kernel-parameters'

## [1.54.1-beta.98] - 2025-03-24

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.97] - 2025-03-22

### Features

- Command 'apply template' not accepts 'limit' as a mandatory argument
- Command 'delete session' has a new argument 'assume-yes' so the command can run unattended
- Add group to audit messages
- Add 'group' to audit messages

## [1.54.1-beta.96] - 2025-03-17

### Features

- Integrate homebrew-tab eth-cscs/homebrew-tap

## [1.54.1-beta.95] - 2025-03-17

### Features

- Integrate homebrew-tab eth-cscs/homebrew-tap

## [1.54.1-beta.94] - 2025-03-17

### Features

- Integrate homebrew-tab eth-cscs/homebrew-tap

## [1.54.1-beta.92] - 2025-03-12

### Bug Fixes

- Rename enum fields in manta config

### Features

- Vault login path is now customized with the 'site_name'

### Miscellaneous Tasks

- Cargo fix
- Update Cargo.toml

## [1.54.1-beta.91] - 2025-03-10

### Features

- Update Cargo.toml

### Miscellaneous Tasks

- Update Cargo.toml

## [1.54.1-beta.90] - 2025-03-06

### Documentation

- Updatae README

## [1.54.1-beta.89] - 2025-03-06

### Features

- Allow 'pa_admin' users deleting 'generic sessions'

### Miscellaneous Tasks

- Udpate Cargo.toml

### Refactor

- Clean code

## [1.54.1-beta.88] - 2025-03-05

### Bug Fixes

- Get session logs now validates the user has access to the CFS session is working on

### Features

- Clean keycloak roles
- Parametrisize gitea url based on 'site name'

### Miscellaneous Tasks

- Clean code
- Code housekeeping
- Update mesa version
- Update Cargo.toml

## [1.54.1-beta.87] - 2025-03-02

### Miscellaneous Tasks

- Remove vault path and vault role id

## [1.54.1-beta.86] - 2025-02-28

### Features

- Power off and power reset '--force' is now the default

## [1.54.1-beta.85] - 2025-02-27

### Miscellaneous Tasks

- Adapt interfaces for new vault authentication

## [1.54.1-beta.84] - 2025-02-27

### Features

- Manta-vault authentication through keycloak token

### Miscellaneous Tasks

- Remove arguments in cli commands not supported by APIs

### Refactor

- Improve error messages

## [1.54.1-beta.83] - 2025-02-25

### Bug Fixes

- Cargo local path dependencies

## [1.54.1-beta.82] - 2025-02-25

### Bug Fixes

- Can't search boot parameters by kernel, initrd or params

### Features

- New functionalities add/update/delete boot parameters
- New functionalities get/add/update/delete redfish endpoint

## [1.54.1-beta.81] - 2025-02-24

### Features

- Add new command to delete a node

### Refactor

- Move code related to add a node to its own module

## [1.54.1-beta.80] - 2025-02-23

### Features

- Delete group command can be force and bypass the orphan node validation

## [1.54.1-beta.79] - 2025-02-23

### Bug Fixes

- Update github pipeline

## [1.54.1-beta.78] - 2025-02-23

### Refactor

- Manta config data

## [1.54.1-beta.77] - 2025-02-23

### Refactor

- Clean code

## [1.54.1-beta.75] - 2025-02-23

### Bug Fixes

- SAT processing fails when watching CFS sessions logs because the process won't wait the CFS session to finish
- Method to get node and cluster hardware components

## [1.54.1-beta.74] - 2025-02-22

### Bug Fixes

- Mesa issue with get hsm group

## [1.54.1-beta.73] - 2025-02-22

### Bug Fixes

- Function argument misalignment

## [1.54.1-beta.72] - 2025-02-21

### Bug Fixes

- Keep genericwa CFS sessions when filtering by hsm or xname

## [1.54.1-beta.71] - 2025-02-21

### Bug Fixes

- Cargo.toml file

## [1.54.1-beta.68] - 2025-02-18

### Features

- Add new backend command 'apply_hw_cluster_pin'

## [1.54.1-beta.67] - 2025-02-17

### Refactor

- Disable code migrated to backend dispatcher

## [1.54.1-beta.66] - 2025-02-16

### Features

- Migrate commands apply session and get configuration to backend dispatcher

## [1.54.1-beta.65] - 2025-02-15

### Features

- Migrate features in add group command from main to 1.5 branches

### Miscellaneous Tasks

- Remove build script
- Get rid of build script

## [1.54.1-beta.64] - 2025-02-14

### Features

- Add shell autocomplete hints in cli
- Implement interfaces to get session and get session log stream

## [1.54.1-beta.63] - 2025-02-09

### Features

- Clean log messages

## [1.54.1-beta.62] - 2025-02-09

### Bug Fixes

- Disable x86_64-unknown-linux-musl in github workload untill we fix the kafka dependency to musl-gcc

## [1.54.1-beta.61] - 2025-02-09

### Features

- Add kafka audit

## [1.54.1-beta.59] - 2025-02-09

### Bug Fixes

- Dependencies

## [1.54.1-beta.58] - 2025-02-09

### Bug Fixes

- Manta log command not working with group or session names

### Features

- Update dependencies

## [1.54.1-beta.57] - 2025-02-08

### Features

- Command 'manta log' not accepts nid, xname, group name or session name

## [1.54.1-beta.56] - 2025-02-03

### Features

- Add autocomplete command

## [1.54.1-beta.55] - 2025-02-03

### Refactor

- Clean code

## [1.54.1-beta.54] - 2025-02-03

### Features

- Migrate code to backend

## [1.54.1-beta.53] - 2025-02-02

### Features

- Improve error management

## [1.54.1-beta.52] - 2025-02-02

### Refactor

- Update mesa version with better error management

## [1.54.1-beta.51] - 2025-02-02

### Refactor

- Update mesa version with better error management

## [1.54.1-beta.50] - 2025-02-02

### Refactor

- Improve error management

## [1.54.1-beta.49] - 2025-02-02

### Features

- Add new Error type to catch 'console errors'

## [1.54.1-beta.48] - 2025-02-01

### Refactor

- Hsm hardware inventory
- Hsm hardware inventory
- Hsm hardware inventory

## [1.54.1-beta.46] - 2025-02-01

### Refactor

- Interfaces

## [1.54.1-beta.45] - 2025-01-30

### Refactor

- Cargo fix

## [1.54.1-beta.41] - 2025-01-29

### Refactor

- Group trait methods
- Group trait methods
- Organize traits
- Organize traits

## [1.54.1-beta.40] - 2025-01-28

### Bug Fixes

- Update github workflow

## [1.54.1-beta.39] - 2025-01-28

### Features

- Update cargo dist workspace
- Console node command now accepts nid

### Refactor

- Clean code

## [1.54.1-beta.38] - 2025-01-27

### Features

- Improve user node input management (nid/xname as comma separated list, hostlist or regex) and migrate this functionality to add and remove nodes to group commands
- Improve user node input management (nid/xname as comma separated list, hostlist or regex) and migrate this functionality to add and remove nodes to group commands

### Refactor

- Clean code

## [1.54.1-beta.37] - 2025-01-26

### Bug Fixes

- List hardware inventory of a node

## [1.54.1-beta.36] - 2025-01-26

### Bug Fixes

- Command 'config show' won't panic if backend API is unrecheable

## [1.54.1-beta.34] - 2025-01-23

### Features

- Power commands now accepts nid nodes
- Migrate code related to translate nid to xnames to bakcends

## [1.54.1-beta.32] - 2025-01-20

### Bug Fixes

- Add hardware inventory mandatory fields

## [1.54.1-beta.31] - 2025-01-18

### Refactor

- Upgrade cicd pipeline

## [1.54.1-beta.30] - 2025-01-18

### Features

- Update github pipeline

## [1.54.1-beta.28] - 2025-01-18

### Features

- Update github pipeline

## [1.54.1-beta.26] - 2025-01-10

### Bug Fixes

- Add backend function add_nodes

## [1.54.1-beta.25] - 2025-01-10

### Features

- Add support for HSM components

## [1.54.1-beta.24] - 2025-01-08

### Features

- Migrate function to get hardware components of a node to backend dispatcher

### Refactor

- Clean code
- Rename modules struct to types

## [1.54.1-beta.23] - 2025-01-06

### Bug Fixes

- Fix type conversion and xname deletion

## [1.54.1-beta.22] - 2025-01-06

### Features

- Migrate hsm functions to backend dispatcher

### Refactor

- Clean code

## [1.54.1-beta.21] - 2025-01-04

### Bug Fixes

- Power reset cluster cli command missing output argument

## [1.54.1-beta.20] - 2025-01-04

### Bug Fixes

- Migrate hsm validation function calls from mesa crate to backend dispatcher

### Features

- Update mesa version

## [1.54.1-beta.17] - 2025-01-02

### Features

- Get group details

## [1.54.1-beta.16] - 2025-01-02

### Features

- Add commands to add and delete a group

## [1.54.1-beta.15] - 2024-12-31

### Bug Fixes

- Update rust toolchain from 1.78.0 to 1.81.0

## [1.54.1-beta.13] - 2024-12-31

### Bug Fixes

- Update github image from ubuntu 20.04 to ubuntu 24.04 and rust toolchain from 1.78.0 to 1.81.0

## [1.54.1-beta.12] - 2024-12-31

### Bug Fixes

- Update github image from ubuntu 20.04 to ubuntu 24.04

## [1.54.1-beta.11] - 2024-12-31

### Bug Fixes

- Update rust toolchain from 1.78.0 to 1.81.0

## [1.54.1-beta.10] - 2024-12-31

### Bug Fixes

- Update cargo dist

## [1.54.1-beta.7] - 2024-12-31

### Bug Fixes

- Mesa library

## [1.54.1-beta.6] - 2024-12-31

### Bug Fixes

- House keeping
- SAT file schema for images section

### Features

- Add static enum dispatch to integrate with business layer
- Integrate functionatlity to integrate boot image with backend dispatcher
- Use backend-dispatcher and ochami-rs as crates
- Use backend-dispatcher and ochami-rs as crates
- Update mesa version

## [1.54.1-beta.5] - 2024-12-06

### Features

- Integrate power and boot operations with CSM backend

## [1.54.1-beta.3] - 2024-12-01

### Features

- Integrate power management operations with IaaS traits
- Update mesa version

### Refactor

- Clean code

## [1.54.1-beta.2] - 2024-12-01

### Refactor

- Clean code

## [1.54.1-beta.1] - 2024-11-30

### Refactor

- Clean code
- Clean code
- Clean code

## [1.53.21] - 2024-11-12

### Features

- Set kernel parameters was changing the kernel value

## [1.53.20] - 2024-11-11

### Refactor

- Cfs_configuration.branch and cfs_configuration.tag are now based on a list of values a specific commit may be related to

## [1.53.19] - 2024-11-11

### Refactor

- Cfs_configuration.branch and cfs_configuration.tag are now based on a list of values a specific commit may be related to

## [1.53.18-alpha.1] - 2024-11-08

### Refactor

- Clean code/modules

## [1.53.17] - 2024-11-07

### Bug Fixes

- Update mesa to fix bug with apply sat command

## [1.53.15] - 2024-11-06

### Bug Fixes

- Argument 'limit' in apply template subcommand should not be mandatory

## [1.53.14] - 2024-11-05

### Features

- Apply sat-file command can now show logs when creating images

## [1.53.13] - 2024-11-04

### Miscellaneous Tasks

- Update cicd pipleine

## [1.53.11] - 2024-11-04

### Bug Fixes

- Add debug messages with rendering jinja templates

### Features

- Update mesa

## [1.53.10] - 2024-10-31

### Features

- Improve performance in get cluster command

## [1.53.9] - 2024-10-31

### Refactor

- Clean code

## [1.53.8] - 2024-10-28

### Refactor

- Fix lint warning messages

## [1.53.7] - 2024-10-28

### Features

- Update mesa

## [1.53.6] - 2024-10-28

### Bug Fixes

- Fail in validating HSM group user has access to

## [1.53.5] - 2024-10-28

### Bug Fixes

- Fetch commit id details

## [1.53.4] - 2024-10-28

### Bug Fixes

- Compilation error

## [1.53.3] - 2024-10-28

### Features

- Add new command 'delete images'

## [1.53.2] - 2024-10-27

### Bug Fixes

- Update mesa to integrate CFS sessions type dynamic creation:wa

### Features

- Add argument 'ansible-playbook-name' to command 'apply session'

## [1.53.1] - 2024-10-25

### Bug Fixes

- Version number

## [1.53.0] - 2024-10-25

### Feature

- Answer yes to questions during apply sat file. (#90)

### Features

- Add log level information to command config show
- Images containing 'generic' in their names are now available to all users

## [1.52.1] - 2024-10-18

### Features

- Update mesa to wait CFS sessions longer

## [1.51.3] - 2024-10-14

### Features

- 'get template' command now prints data in json format

## [1.51.2] - 2024-10-14

### Bug Fixes

- Update mesa

## [1.51.1] - 2024-10-14

### Bug Fixes

- Improve output message

## [1.51.0] - 2024-10-14

### Features

- Add new command 'apply template' to crate a new BOS session from a BOS sessiontemplate

## [1.50.18] - 2024-10-14

### Features

- Prepare HSM goup operations for next version

## [1.50.17] - 2024-10-11

### Features

- Migration node command now accepts a hostlist as list of input nodes

### Refactor

- Update cli docs

## [1.50.16] - 2024-10-04

### Features

- Migrate to CFS configuration v3

## [1.50.14] - 2024-10-03

### Bug Fixes

- Set kernel parameters

## [1.50.13] - 2024-10-03

### Bug Fixes

- Argument parsing in 'power on cluster' command

## [1.50.12] - 2024-10-01

### Features

- Update mesa version

## [1.50.11] - 2024-10-01

### Refactor

- Clean cli commands

## [1.50.9] - 2024-09-29

### Refactor

- Clean code

## [1.50.7] - 2024-09-28

### Refactor

- Apply_sat_file code

## [1.50.6] - 2024-09-27

### Bug Fixes

- Error checking cli help if manta not fully configured

## [1.50.4] - 2024-09-27

### Bug Fixes

- Fix unit tests
- Unit tests
- Imports

### Refactor

- Organise modules

## [1.50.3] - 2024-09-23

### Bug Fixes

- Command 'get kernel-parameters' for a cluster combined with filter not grouping hsm groups correctly

## [1.50.0] - 2024-09-18

### Features

- New command 'get nodes' to query a list of xnames from different HSM groups

## [1.49.5] - 2024-09-18

### Features

- Command 'get cluster' sorts the HSM groups each node belongs to

## [1.49.4] - 2024-09-18

### Features

- 'get cluster' command now displays the list of HSM groups in multiple lines to make better use of screen real estate

## [1.49.3] - 2024-09-18

### Features

- Add HSM group name in 'get cluster' command output

## [1.49.2] - 2024-09-17

### Features

- Improve performance when running command "get cluster"

## [1.49.1] - 2024-09-16

### Features

- Subcommand get kernel parameters group kernel parameters by xnames

## [1.49.0] - 2024-09-09

### Features

- Update mesa version

## [1.47.2] - 2024-09-05

### Bug Fixes

- Improve cli interface of 'get kernel-parameters' subcommand

## [1.47.1] - 2024-08-25

### Bug Fixes

- Update Cargo.toml

## [1.47.0] - 2024-08-25

### Features

- Add pcs utils
- Power management commands now accept a new argument 'output' to change the output format

## [1.46.20] - 2024-08-22

### Refactor

- Rename get kernel-parameters argument

## [1.46.19] - 2024-08-22

### Bug Fixes

- But in set subcommand

### Refactor

- Clean code

## [1.46.18] - 2024-08-21

### Features

- Improve functionality to stop a cfs session
- Stop running session checks is session to stop is actually running, otherwise, it gracefulyl stops

## [1.46.17] - 2024-08-16

### Bug Fixes

- Improve error management when processing SAT files

## [1.46.16] - 2024-08-15

### Bug Fixes

- Fix issue when changing runtie configuration would trigger manta asking user confirmation to reboot the nodes

## [1.46.15] - 2024-08-12

### Refactor

- Migrate code to migrate nodes between hsm groups to mesa

### Fis

- Bug when creating manta config file and CA root public cert file does not exists

## [1.46.14] - 2024-08-11

### Features

- Filter sat file template data base on cli arguments
- Apply sat file can now filter by image or sessiontemplate

## [1.46.13] - 2024-08-04

### Bug Fixes

- Get session table showing formatted stated time in status cell

### Features

- Print config in log debug

## [1.46.12] - 2024-08-03

### Bug Fixes

- Cli hsm argument has preference vs hsm in config file
- Log command ignores default hsm group and checks CFS session is linked to any HSM group the user has access to

### Features

- Cli won't hide hsm-group arguments if default hsm has been setup

### Refactor

- Rename test file
- Code housekeeping

## [1.46.11] - 2024-07-31

### Features

- Format datetime when listing configurations and images

## [1.46.10] - 2024-07-31

### Bug Fixes

- Authentication bug

### Refactor

- Improve cli help text

## [1.46.9] - 2024-07-31

### Refactor

- Add aliases to help command

## [1.46.8] - 2024-07-31

### Features

- Update mesa library

## [1.46.7] - 2024-07-30

### Features

- Update mesa

## [1.46.6] - 2024-07-30

### Bug Fixes

- Update mesa version

## [1.46.5] - 2024-07-30

### Bug Fixes

- Config autogenerator allows to provide an empty socks5 proxy value
- Update mesa version

## [1.46.4] - 2024-07-29

### Bug Fixes

- Config param  will search for either full path or file inside /home/msopena/.config/manta/

## [1.46.3] - 2024-07-29

### Features

- Add new command  to get the list of kernel parameters for a list of nodes or a cluster
- New argument in  command to filter the list of kernel parameters listed

## [1.46.2] - 2024-07-29

### Refactor

- Housekeeping code managing config file

## [1.46.0] - 2024-07-29

### Features

- Config file autogeneration
- Config file autogeneration

## [1.45.3] - 2024-07-28

### Features

- Update mesa version

## [1.45.1] - 2024-07-26

### Bug Fixes

- Bug managing urls in config file

## [1.45.0] - 2024-07-26

### Refactor

- Clean config file
- Clean config file

## [1.43.0] - 2024-07-23

### Features

- Copy ansible templating functionality and session vars file is both a ninja template and a values file, 'manta apply sat' will render the values file with itself

## [1.42.3] - 2024-07-12

### Bug Fixes

- Move deprecated messages in command get nodes to log when output is json

## [1.42.2] - 2024-07-12

### Bug Fixes

- Move deprecated messages in command get nodes to log when output is json

## [1.42.1] - 2024-07-11

### Features

- Apply sat now accepts ansible_passthrough argument as env var

## [1.41.6] - 2024-07-07

### Bug Fixes

- Workaround system hsm groups filtering

### Features

- Get sessions related to xnames

## [1.41.5] - 2024-07-05

### Bug Fixes

- Error management when any HSM group in JWT token does not exists

## [1.41.4] - 2024-07-05

### Bug Fixes

- Update mesa vesion to fix local repo validation bug

## [1.41.3] - 2024-07-05

### Bug Fixes

- Update cargo dist and stdout logs

## [1.41.2] - 2024-07-04

### Bug Fixes

- Fix CICD error by mesa dependency in Cargo.toml

## [1.41.1] - 2024-07-04

### Refactor

- Clean code

## [1.41.0] - 2024-07-03

### Refactor

- Update documentation

## [1.40.0] - 2024-07-03

### Bug Fixes

- Improve cli help
- Migrate from BOS v1 to BOS v2
- Arggroup bug
- Fix import
- IMS job creation returns CSM error msg is request failt

### Features

- Add cli command
- New env var MANTA_CONFIG to set the path for the configuration file
- Update mesa version

### Refactor

- Clean code

## [1.38.1] - 2024-06-28

### Bug Fixes

- Improve deprecated messages

## [1.38.0] - 2024-06-28

### Bug Fixes

- Update mesa version

### FEAT

- Command 'apply hw cluster' now can reuse nodes in 'target' HSM group

### Features

- Integrate "pin" and "unpin" features to "apply hw cluster" command

### Refactor

- Clean code
- Code housekeeping
- Rename apply_hw_cluster modules according to pin and unpin strategy

## [1.36.3] - 2024-06-12

### FEAT

- Update manta version

### Refactor

- Clean code

## [1.36.2] - 2024-06-09

### REFACTOR

- Fix subcommands

## [1.36.1] - 2024-06-02

### Bug Fixes

- Use new mesa library to fix issue getting commit id details form gitea

## [1.36.0] - 2024-06-02

### Bug Fixes

- Cli help

## [1.35.8] - 2024-05-29

### Bug Fixes

- Enable openssl-vendores feature got git2 crate to avoid breaking apple images during CI/CD pipeline
- Update boot parameters

## [1.35.7] - 2024-05-28

### Bug Fixes

- Try to fix ci/cd pipeline building openssl-sys

## [1.35.6] - 2024-05-28

### Bug Fixes

- Init cargo dist

## [1.35.5] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

## [1.35.4] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

## [1.35.3] - 2024-05-28

### Bug Fixes

- Downgrade cargo-dist in CI pipeline

## [1.35.2] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

## [1.35.0] - 2024-05-23

### Features

- New command 'get cluster <cluster name> --output summary'

## [1.33.0] - 2024-05-21

### Features

- Get configuration with details now shows CFS configuration derivatives (CFS sessions, BOS sessiontemplate and IMS images related to a CFS configuration)

## [1.32.5] - 2024-05-20

### Features

- Update hsm group members
- Update mesa version
- Update mesa version
- Update mesa version

### Refactor

- Change var names

## [1.32.3] - 2024-05-02

### Features

- Refactor template output information

## [1.32.1] - 2024-05-01

### Features

- Filter image by id
- Update mesa version

### Refactor

- Clean output of command validate-local-repo

## [1.31.2] - 2024-04-30

### Features

- Format data in manta get configuration -n

## [1.29.5] - 2024-04-17

### Features

- Print most recent CFS session logs related to a cluster

### Refactor

- Fix mesa library location

## [1.29.4] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

## [1.29.3] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

## [1.29.2] - 2024-04-16

### Bug Fixes

- Bug filtering CFS sessions through HSM groups
- Fix bug filtering bos sessiontemplate by HSM group
- Print BOS sessiontemplate information properly by removing the type column in table

### Features

- Add functionality to filter CFS sessions by state

## [1.28.14] - 2024-03-17

### Refactor

- Move tests to /test/ directory

## [1.28.13] - 2024-03-15

### Features

- Add selection prompmt to delete auth token

## [1.28.12] - 2024-03-15

### Bug Fixes

- Config unset hsm command

## [1.28.11] - 2024-03-15

### Bug Fixes

- Format cfs layer data and clean stoud log traces

## [1.28.9] - 2024-03-14

### Features

- Handle auth tokens for multiple sites at the same time

## [1.28.8] - 2024-03-13

### Bug Fixes

- BUG SAT file session_template validation ignoring previous SAT file version

## [1.28.7] - 2024-03-12

### Features

- Improve SAT file validation to improve user feedback
- Test new cc to build apple target binaries

## [1.28.6] - 2024-03-04

### Bug Fixes

- Remove apply artifacts/targets from CI pipeline

## [1.28.3] - 2024-03-01

### Features

- Prepare to substitute apply configuration, apply image and apply cluster to apply sat-file

## [1.28.2] - 2024-03-01

### Bug Fixes

- Test apple artifacts

## [1.28.1] - 2024-03-01

### Features

- Update manta version

## [1.27.0] - 2024-02-28

### Features

- Update manta version

## [1.26.0] - 2024-02-25

### Features

- Get hw components subcommands now can print information as a summary of all hw components in a cluster

## [1.25.1] - 2024-02-25

### Refactor

- Reformat how CFS configuration layer details are printed on screen

## [1.24.2] - 2024-02-23

### Bug Fixes

- Disable apple targets due to a bug in cc crate used by openssl crate

## [1.24.0] - 2024-02-22

### Bug Fixes

- Mesa library
- Manta version

### Features

- Initial woking state
- Improve the function that merges 2 yaml structs by avoiding having to rewrite siblings

## [1.23.0] - 2024-02-20

### Features

- New feature to use the SAT files as jinja2 templates (#37)
- Update mesa version

## [1.22.11] - 2024-02-20

### Bug Fixes

- Bug with manta panicking while creating a cluster if image creation fails
- Error when getting CFS session logs using a CFS session which does not exists
- Update mesa library

## [1.22.10] - 2024-02-17

### Bug Fixes

- Command update node fails if user is not restricted to any HSM groups
- Fix error parsing cli opn 'ansible-verbosity' to 'apply image' subcommand

## [1.22.9] - 2024-02-16

### Bug Fixes

- Show_config function breaks if the list of HSM groups the user has access to is empty

## [1.22.8] - 2024-02-15

### Features

- Update mesa version

## [1.22.7] - 2024-02-13

### Bug Fixes

- Fix stdout messages
- Manta crashes:wa when CFS configuration layer had no commit id

## [1.22.6] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

## [1.22.5] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

## [1.22.4] - 2024-02-13

### Features

- Update mesa version

## [1.22.3] - 2024-02-10

### Bug Fixes

- Mesa crate

## [1.22.2] - 2024-02-10

### Bug Fixes

- Apply cluster command failing if session_template section in SAT file was in old format

## [1.22.1] - 2024-02-10

### Refactor

- Clean code and stdout messages

## [1.22.0] - 2024-02-09

### Bug Fixes

- Create bos sessiontemplate from SAT file

### Features

- Update manta version

### Refactor

- Clean gitea code since it is moved to mesa
- Clean code
- Clean code
- Create module for SAT code

## [1.21.3] - 2024-01-30

### Bug Fixes

- Bos sessiontemplate filter by list of xnames

### Features

- Update mesa version

## [1.21.2] - 2024-01-29

### Bug Fixes

- Format errors when deleting an image which does not exists

## [1.21.1] - 2024-01-29

### Bug Fixes

- Error when deleting an image based on a db recod but the artifact does not exists

## [1.21.0] - 2024-01-29

### Features

- Add new param to apply cluster to avoid nodes from rebooting

## [1.20.35] - 2024-01-28

### Bug Fixes

- Update mesa version in cargo.toml

## [1.20.34] - 2024-01-28

### Fixes

- Migrate backup and migrate restore (#11)

### Refactor

- Code checkif user has access to HSM groups and members
- Fix some log messages

## [1.20.33] - 2024-01-27

### Bug Fixes

- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add releases for other OS
- Add releases for other OS
- Remove windows as a target

### Features

- Add openssl vendor feature to git2

## [1.20.6] - 2024-01-26

### Features

- Sort hsm available list in 'config show' command
- Add new target for mac, the idea is to have a new binary in github releases for mac users

## [1.20.5] - 2024-01-24

### Bug Fixes

- Update mesa version

## [1.20.4] - 2024-01-24

### Bug Fixes

- Update mesa version to fix a bug

## [1.20.3] - 2024-01-24

### Bug Fixes

- Bug in 'apply cluster' subcommand where it was filtering wrong images

## [1.20.2] - 2024-01-24

### Bug Fixes

- Add migrate subcommand
- Merge migration functionality

### Refactor

- Apply clippy suggestions

## [1.20.0] - 2024-01-22

### Bug Fixes

- Simplify the collection of the HSM group description data.
- Merge cluster migration branch
- Cli build code fix

### Feature

- Migrate/backup first commit (partial)
- Migrate/backup ignore JetBrains stuff
- Migrate/backup download all files of a bos session template
- Migrate/backup fix count of artifacts in download info
- Migrate/backup add support to produce a file with the list of xnames belonging to the HSM groups in the BOS session template.
- Migrate/backup cleanup
- Migrate/backup more cleanup
- Migrate/restore load backed files into memory

## [1.19.3] - 2024-01-21

### Refactor

- Clean code

## [1.19.2] - 2024-01-21

### Bug Fixes

- Rollback apply hw so it unpins all nodes in target hsm
- Fix issues related to add hw and remove hw subcommands

## [1.19.1] - 2024-01-19

### Bug Fixes

- Fix bug when creating clusters using sat file

### Features

- Add and remove nodes from HSM group
- Add new mesa version

### Refactor

- Clean code related to subcommand 'apply hw'
- Clean code
- Add apply hw-configuration cli help message
- Clippy fixes
- Clippy fixes
- Clippy fixes

## [1.19.0] - 2024-01-15

### Features

- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)
- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)

## [1.18.0] - 2024-01-11

### Bug Fixes

- Disable tests which need to connect to csm apis becuase they are not accessible from github test environment

## [1.17.0] - 2024-01-11

### Refactor

- Get mesa from repo

## [1.16.2] - 2024-01-10

### Bug Fixes

- Apply session and update mesa library

## [1.16.1] - 2024-01-10

### Features

- Remove hw components from a target hsm groups and node scores calculated based on scarcity across target and parent hsm groups
- Apply and remove working with simple examples, not fully tested but in good condition
- Add new hw components to a cluster

### Refactor

- Clean code
- Refactor code

## [1.16.0] - 2024-01-04

### Features

- Get hw cluster now accepts a new 'pattern' output

### Refactor

- : add clippy suggestions

## [1.15.0] - 2024-01-01

### Bug Fixes

- Replace std sleep for tokio sleep

### Refactor

- Cargo fmt
- Use new manta utility functions
- Adapt to new mesa code
- Adapt to new mesa code
- Adopt mesa changes
- Adapt to mesa code
- Housekeeping around HSM module
- Adapt to mesa code
- Adapt to new mesa code
- Adapt to new mesa code
- Swap to mesa library

## [1.14.0] - 2023-12-25

### Refactor

- Clean code
- Adapt code to new mesa

## [1.13.5] - 2023-12-22

### Refactor

- Update mesa version

## [1.13.3] - 2023-12-21

### Features

- Update mesa version

### Refactor

- High refactoring
- Cfs configuration structs
- Rename method name to get multiple CFS components
- Hoursekeeping around node methods

## [1.13.2] - 2023-12-11

### Refactor

- Clean code and update mesa version
- Cargo fmt

## [1.13.1] - 2023-12-10

### Refactor

- Refactor code to new mesa method signatures

## [1.12.12] - 2023-12-08

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.11] - 2023-12-08

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.10] - 2023-12-08

### Bug Fixes

- Fix bug with get configuration subcommand

## [1.12.6] - 2023-11-16

### Bug Fixes

- Bug getting cluster with nodes being configured

## [1.12.5] - 2023-11-16

### Bug Fixes

- Fix bug getting hsm group from cli param

## [1.12.4] - 2023-11-16

### Bug Fixes

- Fix 'get cluster status' sub command

### Refactor

- Fix 'get cluster' command help message typo
- Add deprecated message in 'get nodes' subcommand
- Fix 'get hsm' cli help message

## [1.12.3] - 2023-11-16

### Documentation

- Fix README

## [1.12.2] - 2023-11-16

### Documentation

- Update README

## [1.10.6] - 2023-11-10

### Refactor

- Code related to cfs session logs

## [1.10.5] - 2023-11-07

### Bug Fixes

- Fix bug where app did not read socks 5 information

## [1.10.4] - 2023-11-07

### Documentation

- Update README with instructions on how to create releases and commit messages best practices for CHANGELOG.md

## [1.10.3] - 2023-11-01

### Bug Fixes

- Add git cliff configuration to support multiline git commits

### Features

- Add subcommand to change log level

## [0.5.1] - 2023-06-21

<!-- generated by git-cliff -->
