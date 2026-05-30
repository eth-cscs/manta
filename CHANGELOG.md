# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Features

- `[server].listen_address` and `[server].port` are now optional. When omitted (and no `--listen-address` / `--port` CLI flag is passed), the server falls back to `0.0.0.0` and to port `8443` if TLS (`cert` + `key`) is configured, else `8080`. Matches the common dev / sidecar setup where TLS is terminated upstream.
- `manta power on/off/reset` now drives the PCS-transition polling loop client-side. `POST /power` returns immediately with the transition id; the CLI polls the new `GET /power/transitions/{id}` snapshot endpoint until the transition reports `completed`, rendering progress on every poll. Eliminates the long-held server connections that used to risk hitting the request timeout on cluster-wide transitions. A new `--no-wait` flag on every `power` subcommand returns the transition id and exits immediately.
- Configurable HTTP timeout. New `[server].request_timeout_secs` (default 60, applies to every route) in `server.toml`. The earlier `power_timeout_secs` knob is gone — with the client-side polling loop it's no longer needed.
- `manta apply sat-file` builds an in-memory execution plan (configurations → images topologically sorted by `base.image_ref` → session_templates) and validates cross-references (no dangling `image_ref`, no cycles) client-side before any HTTP call.
- The `--image-only` / `--sessiontemplate-only` filter logic moved into the plan builder; the old `apply_sat_file_filters` helper is removed from `manta-shared`.
- The CLI now dispatches the SAT plan one element at a time via three new server endpoints — `POST /sat-file/configurations`, `POST /sat-file/images`, `POST /sat-file/session-templates` — accumulating a `ref_name → image_id` lookup between calls so chained images and session_templates resolve. The user-visible four-list summary is unchanged.

### Bug Fixes

- `impl SatTrait for StaticBackendDispatcher` now forwards the three new per-element methods through the `dispatch!` macro; without this they would have fallen through to the trait's default "not implemented" impls at runtime.

### Compatibility

- `POST /sat-file` (whole-file) is retained for SAT files with a `hardware:` section while the per-element flow only covers configurations, images, and session_templates. A follow-up will either add a per-element hardware endpoint or drop hardware from `manta apply sat-file`.

## [2.0.0-beta.15] - 2026-05-27

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

### Fox

- Cargo.toml files

<!-- generated by git-cliff -->
