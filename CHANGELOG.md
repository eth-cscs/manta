# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking changes

- The CLI no longer emits Kafka audit events. The `[auditor.kafka]` block in `cli.toml` is no longer read (silently ignored if present, since `CliConfiguration` does not derive `deny_unknown_fields`). Server-side audit coverage — every request via `log_requests` middleware + per-`/auth/*` events via `send_auth_audit` — supersedes the previous CLI-emitted stream since every CLI action goes through HTTP and is therefore already recorded server-side. `common::audit::maybe_send_audit` is gone; `common::kafka::Kafka` is no longer constructed by the CLI; the `kafka_audit_opt` field on `AppContext` and the `auditor` field on `CliConfiguration` are removed.

## [2.0.0-beta.16] - 2026-05-30

### Bug Fixes

- `add node -d/--disabled` actually disables the node
- Update cli_tests.rs after the manta-cli -> manta binary rename

### Build

- Convert to Cargo workspace with crates/manta-cli as the sole member
- Extract manta-shared as a library crate
- Move backend dispatcher into manta-shared
- Move common/ into manta-shared
- Extract manta-server as a separate binary crate
- Write generated man pages and completions to OUT_DIR
- Per-crate license-file declarations to fix cargo-dist asset copy
- Rename binary to 'manta'
- Per-crate Dockerfiles for manta-cli + manta-server
- Bump builder image to rust:1.88-bookworm
- Bump direct deps to clear cargo-audit advisories

### Documentation

- Document Cargo workspace split + scope CI fmt/grep paths
- Finish Phase 1 of rustdoc — sat_file + config types + 4 doctests + CI
- Refresh module headers + fn docstrings after Tier 3.2 renames
- Refresh user docs, rustdoc, and pin pure helpers

### Features

- Add GET /groups/available + /groups/all endpoints
- Expand client-authentication tracing
- Log every HTTP request as a copy-pasteable curl command
- Route add_* commands through output::action_result
- Route delete_* commands through output::action_result
- Route update/apply/migrate commands through output::action_result
- Route config/power commands through output::action_result
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
- Show timestamps in server log output
- Build an ordered execution plan in `manta apply sat-file`
- Dispatch the execution plan element-by-element
- Configurable request and per-route /power timeouts
- Move PCS-transition polling loop to the CLI

### Miscellaneous Tasks

- Untrack stray runtime config.toml; ignore /crates/*/config.toml
- Add LICENSE file to creates
- Update Cargo.toml
- Bump csm-rs to 1.0.0-beta.2

### Refactor

- Remove per-site manta_server_url field
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
- Collapse MantaClient query-building into QueryBuilder
- Split build.rs (1311 LOC) per command family
- Decouple from csm-rs / ochami-rs / manta-backend-dispatcher
- Flatten AppContext — drop CliInfra/CliConfig wrappers
- Slim CliConfiguration and drop dead Site vault fields
- Move backend bridge from manta-shared to manta-server
- Split http_client.rs (1254 LOC) into per-resource modules
- Collapse the two crate::common re-export shims
- Use get_flag for `add node --disabled`
- Silence struct_excessive_bools on 4 audited structs
- Add ArgMatchesExt to dedupe arg-extraction boilerplate
- Rename --hsm-group to --group with backwards-compat alias
- Pluralize redfish-endpoint subcommand for consistency
- Flatten arbitrary command directory splits
- Rename cluster-named command modules (Tier 3.2 phase 5/N)
- [**breaking**] Move render + filter + preview to CLI
- Pipe parsed Value through trait; delete manta-shared SatFile

### Styling

- Cargo fmt baseline after workspace split
- Cargo fmt baseline across 9 files touched in recent commits
- Cargo clippy --fix sweep for cosmetic lints

### Testing

- Cover QueryBuilder and ws_base_url in http_client
- Cover per-element flow and refresh module headers

### Fox

- Cargo.toml files

<!-- generated by git-cliff -->
