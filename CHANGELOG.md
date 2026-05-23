# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Features

- Auth tracing: expand client-side auth chain logs with structured fields, span, and explicit fall-through messages
- Auth tracing: instrument the service layer with site/backend/elapsed fields on every backend call
- Auth tracing: log the dispatcher boundary into csm-rs/ochami-rs with `backend=csm|ochami` so failures attribute to the right client
- Server startup: log the path of the configuration file the server loaded (env var vs default lookup)
- Server startup: emit a summary of effective settings (listen address, port, cert path, log filter, audit, rate limit) and per-site backend URLs as an INFO block; TLS key, SOCKS userinfo, and k8s/vault secrets never logged
- CLI HTTP client: at DEBUG, every outbound request is logged as a copy-pasteable `curl` invocation with `Authorization` and JSON `password`/`token` fields auto-redacted
- CLI output: every mutating command (`add`, `delete`, `update`, `apply`, `migrate`, `power`, `config set/unset`) honours `-o/--output {table,json}` and routes its status through a single `output::action_result` renderer. JSON mode wraps the payload as `{"status":"ok","message":"...","data":...}` for clean script consumption

### Refactor

- Server: replace the 2-line handler preamble (state lookup + infra context) with a one-line `ctx.infra()` borrow, validated up front by the `RequestCtx` extractor
- Server: print the startup configuration summary to stdout instead of the log stream, so operators see it regardless of the `log` filter
- `hw_cluster` scoring: widen `f32`→`f64` and scope `cast_precision_loss` allow with a justifying comment on the two scoring functions
- Silence `struct_excessive_bools` on 4 audited structs (`UpdateRedfishEndpointParams`, `ApplySatFileParams`, `SatApplyOptions`, `PostSatFileRequest`) with rationale comments
- CLI normalization Tier 1: add `ArgMatchesExt` trait (`req_str`/`opt_str`/`opt_string`) and migrate every handler + the 12 commands with the densest direct arg extraction — ~120 LOC removed
- CLI normalization Tier 1: rename `--hsm-group` to `--group` with `--hsm-group` as a visible clap alias, on every subcommand that previously used the long form
- CLI normalization Tier 1: pluralise `redfish-endpoint` → `redfish-endpoints` for `add`, `update`, `delete` with the singular form retained as a visible alias
- CLI normalization Tier 1: flatten three single-purpose command directories (`apply_hw_cluster_pin`, `apply_hw_cluster_unpin`, `hw_cluster_common`) and promote `delete_images/command.rs` to a single `delete_images.rs`

### Documentation

- API: new Troubleshooting section with curl recipes (auth bootstrap, validate, GET/POST/DELETE, SSE log stream, WebSocket console) and a status-code decision table

### Build

- Bump direct dependencies to clear cargo-audit advisories

### Continuous Integration

- Build both Dockerfiles in CI to catch image regressions in PRs

### Style

- `cargo clippy --fix` cosmetic sweep (`uninlined_format_args`, `redundant_closure`, `map_unwrap_or`, etc. across 65 files)

## [2.0.0-beta.11] - 2026-05-22

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

### Documentation

- Document Cargo workspace split + scope CI fmt/grep paths
- Finish Phase 1 of rustdoc — sat_file + config types + 4 doctests + CI

### Features

- Add GET /groups/available + /groups/all endpoints

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

### Styling

- Cargo fmt baseline after workspace split
- Cargo fmt baseline across 9 files touched in recent commits

### Testing

- Cover QueryBuilder and ws_base_url in http_client

### Fox

- Cargo.toml files

<!-- generated by git-cliff -->
