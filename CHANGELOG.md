# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### CLI normalization — deferred follow-ups

Several CLI shapes from the earlier Tier 1/2 plan landed; the items
below close out the remaining Option-A renames from task #60. Every
old form keeps working for one release with a `[DEPRECATED]` tag in
its help text plus a one-line stderr warning on use. Plan: drop the
old forms in the next major release.

| Old form | New canonical form |
|---|---|
| `manta apply session` | `manta run session` |
| `manta apply boot cluster` | `manta apply boot group` |
| `manta apply hardware cluster` | `manta apply hardware group` |
| `manta get cluster` | `manta get group-nodes` |
| `manta get hardware cluster` | `manta get group-hardware` |
| `manta power on cluster` | `manta power on group` |
| `manta power off cluster` | `manta power off group` |
| `manta power reset cluster` | `manta power reset group` |
| `manta migrate vCluster backup` | `manta backup vcluster` |
| `manta migrate vCluster restore` | `manta restore vcluster` |
| `manta add-nodes-to-groups` | `manta add nodes` |
| `manta remove-nodes-from-groups` | `manta delete nodes` |
| `--target-cluster` | `--target-group` |
| `--parent-cluster` | `--parent-group` |
| `--create-hsm-group` | `--create-group` |
| `--delete-hsm-group` | `--delete-group` |
| `--create-target-hsm-group` | `--create-target-group` |
| `--delete-empty-parent-hsm-group` | `--delete-empty-parent-group` |

### Features

- `config show` honours `-o/--output {table,json}` via a dedicated
  `output::config_summary` renderer
- `apply session` and `apply sat-file` route their final result
  through `output::action_result` (`--output json` returns
  `{"status":"ok","message":...,"data":...}`)
- `MantaClient::apply_sat_file` now returns the server's response
  payload (previously discarded)

### Refactor

- Internal source-tree rename — `commands::{get_cluster,
  get_hardware_cluster, apply_hw_cluster, apply_boot_cluster,
  add_hw_component_cluster, delete_hw_component_cluster}` -> their
  `_group`-suffixed counterparts. No user impact.

## [2.0.0-beta.12] - 2026-05-23

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

### Features

- Add GET /groups/available + /groups/all endpoints
- Expand client-authentication tracing
- Log every HTTP request as a copy-pasteable curl command
- Route add_* commands through output::action_result
- Route delete_* commands through output::action_result
- Route update/apply/migrate commands through output::action_result
- Route config/power commands through output::action_result

### Miscellaneous Tasks

- Untrack stray runtime config.toml; ignore /crates/*/config.toml
- Add LICENSE file to creates
- Update Cargo.toml

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

### Styling

- Cargo fmt baseline after workspace split
- Cargo fmt baseline across 9 files touched in recent commits
- Cargo clippy --fix sweep for cosmetic lints

### Testing

- Cover QueryBuilder and ws_base_url in http_client

### Fox

- Cargo.toml files

<!-- generated by git-cliff -->
