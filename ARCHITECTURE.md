# Manta Architecture

This document describes the internal structure of the manta codebase for contributors.

---

## Workspace layout

manta is a Cargo workspace with three member crates:

```
manta/
├── Cargo.toml                       (workspace manifest)
└── crates/
    ├── manta-shared/   (lib)        — wire types, common helpers, backend dispatcher
    ├── manta-cli/      (bin)        — terminal client, depends on manta-shared
    └── manta-server/   (bin)        — Axum HTTPS server + service layer, depends on manta-shared
```

Dep graph: `manta-cli → manta-shared ← manta-server`. Neither binary depends on the other, so each can be built and shipped on its own (`cargo build -p manta-cli` / `cargo build -p manta-server`).

`manta-shared` exposes three top-level modules:

| Module | Used by | Contents |
|--------|---------|----------|
| `shared` | both bins | Wire types (`params/`, `output/`) and pure helpers (`cluster_status`, `node_summary`, …) |
| `common` | both bins | Config loader, JWT ops, Kafka audit producer, authorization helpers, logging |
| `backend_dispatcher` + `manta_backend_dispatcher` | both bins | `StaticBackendDispatcher` enum and trait `impl`s routing to `csm-rs`/`ochami-rs` |

`manta-cli` keeps its CLI-only modules under `crates/manta-cli/src/cli/common/` (e.g. `authentication`, `hooks`, `user_interaction`); `manta-server` keeps its server-only common under `crates/manta-server/src/server/common/` (e.g. `node_ops`, `vault`, `boot_parameters`).

---

## Layer overview

```
User
  │
  ├─ CLI mode (manta-cli binary)
  │    crates/manta-cli/src/cli/          — argument parsing, output tables, user prompts
  │      └─ crates/manta-shared/          — common + shared helpers + backend dispatcher
  │           └─ csm-rs / ochami-rs       — HTTP clients for CSM / OpenCHAMI
  │
  └─ HTTP server mode (manta-server binary)
       crates/manta-server/src/server/    — axum HTTPS server, JWT auth middleware
         └─ crates/manta-server/src/service/ — business logic, orchestration, filtering
              └─ crates/manta-shared/     — common + shared helpers + backend dispatcher
                   └─ csm-rs / ochami-rs
```

Both binaries share `manta-shared`. The CLI does not link the service layer or axum; the server does not link the CLI command tree.

---

## Entry points

Each binary has its own `main.rs`:

### `crates/manta-cli/src/main.rs`

Startup runs in two phases:

1. **Single-threaded phase** — parse CLI args, load `~/.config/manta/config.toml`. The SOCKS5 proxy URL is read from `site.socks5_proxy` in the config.
2. **Multi-threaded phase** — start the tokio runtime, construct a `StaticBackendDispatcher` (passing `socks5_proxy`), and run the requested CLI command.

### `crates/manta-server/src/main.rs`

Mirrors the CLI bootstrap, then starts the HTTPS server. Minimal Clap surface: `--port`, `--cert`, `--key`, `--listen-address`. The `manta serve` subcommand has been removed from the CLI; users invoke `manta-server` directly.

---

## Layer responsibilities

### `crates/manta-cli/src/cli/`

Presentation layer. Responsibilities:

- **`build/`** — Clap command and subcommand definitions.
- **`process/`** — Argument extraction and dispatch to the service layer (via `manta-shared` helpers or HTTP calls through `MantaClient`).
- Output formatting via `comfy-table` for terminal tables.
- Interactive prompts via `dialoguer`.
- Error handling via `anyhow::Error`; CLI handlers terminate with `eprintln!` + `process::exit()`.

CLI code **must not** contain business logic. It calls service functions with typed parameters and formats their results.

### `crates/manta-server/src/service/`

Business logic layer. Modules: `session`, `configuration`, `group`, `node`, `image`, `template`, `boot_parameters`, `kernel_parameters`, `hardware`, `hw_cluster`, `cluster`, `ephemeral_env`, `sat_file`, `migrate`, `redfish_endpoints`.

Each module receives an `&InfraContext<'_>` plus a bearer token and typed parameters, and returns typed results. This layer:

- Orchestrates multi-step operations (e.g. create config → build image → create session).
- Applies filtering, sorting, and business rules.
- Uses `manta_backend_dispatcher::error::Error` (not `anyhow`).
- Has no knowledge of terminal output or HTTP request/response shapes.

### `crates/manta-shared/src/backend_dispatcher/`

Trait implementation glue. Implements all `manta-backend-dispatcher` traits (`CfsTrait`, `GroupTrait`, `BootParametersTrait`, etc.) on `StaticBackendDispatcher` using a `dispatch!` macro. The macro expands to a `match` that routes each method call to either the `Csm` or `OCHAMI` variant.

### `crates/manta-shared/src/manta_backend_dispatcher.rs`

Defines the `StaticBackendDispatcher` enum:

```rust
pub enum StaticBackendDispatcher {
    CSM(Csm),
    OCHAMI(Ochami),
}
```

`StaticBackendDispatcher::new(backend_type, base_url, root_cert, socks5_proxy)` reads the `backend` field from the site config and constructs the appropriate variant, forwarding `socks5_proxy` to both `Csm::new` and `Ochami::new`.

### `crates/manta-shared/src/common/`

Shared utilities used by both CLI and server:

| Module | Purpose |
|--------|---------|
| `config/` | Load and validate `~/.config/manta/config.toml` |
| `jwt_ops` | Decode and validate JWT bearer tokens |
| `kafka` | Fire-and-forget audit event producer |
| `log_ops` | Logger initialisation |
| `authorization` | HSM-group membership checks |
| `app_context` | `InfraContext` + `AppContext` types |
| `audit` | Audit trait + log writer |

CLI-only modules (`authentication`, `hooks`, `user_interaction`, `kernel_parameters_ops`, `local_git_repo`) live under `crates/manta-cli/src/cli/common/`. Server-only modules (`node_ops`, `vault`, `boot_parameters`, `hw_inventory_utils`, `ims_ops`) live under `crates/manta-server/src/server/common/`.

### `crates/manta-server/src/server/`

Axum HTTPS server. Key files:

| File | Purpose |
|------|---------|
| `mod.rs` | `start_server` — binds TLS, builds router, logs to stderr when the socket is ready to accept connections |
| `routes.rs` | Registers ~47 REST endpoints + 2 WebSocket upgrades under `/api/v1/`; serves `GET /openapi.json` and `GET /docs` |
| `handlers.rs` | Per-endpoint functions: extract bearer token, deserialise params, call service, serialise response |
| `api_doc.rs` | `ApiDoc` struct — assembles the OpenAPI 3.0 spec from all `#[utoipa::path]` annotations; adds `bearerAuth` security scheme and `/api/v1` server base path |

`ServerState` (wrapped in `Arc`) owns all infrastructure: backend dispatcher, TLS certificates, optional Vault/k8s URLs.

---

## Context objects

| Type | Used by | Contents |
|------|---------|---------|
| `InfraContext<'_>` | Service layer | Backend dispatcher, base URLs, root CA cert, SOCKS5 proxy, optional vault/k8s URLs |
| `AppContext` | CLI layer | Composes `InfraContext` + `CliConfig` (active HSM group, Kafka audit config, raw settings, `manta_server_url`) |
| `Arc<ServerState>` | HTTP server | Infrastructure behind a reference-counted pointer; each handler calls `.infra_context()` |

`manta_server_url` lives in `CliConfig`, not `InfraContext`, because it is a CLI routing decision (proxy requests through the manta HTTP server instead of calling the backend directly). It is not needed by the service layer or the HTTP server.

---

## Configuration files

Manta reads two TOML files, one per binary:

| Binary | Default path | Env override |
|---|---|---|
| `manta-cli` | `~/.config/manta/cli.toml` | `MANTA_CLI_CONFIG` |
| `manta-server` | `~/.config/manta/server.toml` | `MANTA_SERVER_CONFIG` |

Both schemas share the `[sites.X]` block (backend, URLs, cert paths) but each carries its own non-shared fields:

| Schema | Fields |
|---|---|
| `CliConfiguration` | `log`, `audit_file`, `site` (active), `parent_hsm_group`, top-level `manta_server_url`, `auditor`, `sites` |
| `ServerConfiguration` | `log`, `audit_file`, `[server]` (TLS, listen, console timeout), `auditor`, `sites` |

The server has no notion of an "active" site — it hosts every entry in its `sites` table simultaneously, and clients select per-request via the `X-Manta-Site` header.

Loaders live in `manta-shared::common::config`: `get_cli_configuration()` and `get_server_configuration()`. Both fail fast with `Error::NotFound` if the file is missing; the error message includes a minimal sample and (if `~/.config/manta/config.toml` is detected on disk) a field-by-field migration mapping. There is no auto-create wizard and no migration subcommand.

## Backend selection

```toml
# In cli.toml: pick the active site
site = "cscs_prod"

[sites.cscs_prod]
backend = "csm"             # or "ochami"
shasta_base_url = "https://api.cscs.ch"
root_ca_cert_file = "cscs_root_cert.pem"

[sites.local_ochami]
backend = "ochami"
shasta_base_url = "https://foobar.openchami.cluster:8443"
root_ca_cert_file = "ochami_root_cert.pem"
```

The active site is chosen by `site = "<name>"` at the top level of `cli.toml`, overridable per-invocation with `--site <name>`. `StaticBackendDispatcher::new` reads the `backend` string and constructs `CSM(...)` or `OCHAMI(...)`.

For the HTTP server, every site in `server.toml`'s `[sites.*]` table gets its own `StaticBackendDispatcher` at startup; client requests select between them via `X-Manta-Site`.

---

## CLI mode vs HTTP server mode

| Aspect | CLI | HTTP server |
|--------|-----|-------------|
| Entry point | `cli::process::process_cli` | `server::start_server` |
| Auth source | Environment variable / Vault / stdin | `Authorization: Bearer` header, per request |
| Context type | `AppContext` (owns `CliConfig`) | `Arc<ServerState>` → `infra_context()` |
| Error handling | `eprintln!` + `process::exit()` | JSON `{"error": "..."}` with HTTP status code |
| Output | Terminal tables / stdout | JSON response body |
| Streaming | stdout | SSE (`/sessions/{name}/logs`) or WebSocket (`/nodes/{xname}/console`) |
| Error type | `anyhow::Error` | `manta_backend_dispatcher::error::Error` |

---

## Error handling conventions

Two error types are used depending on context (enforced by CI):

- **`manta_backend_dispatcher::error::Error`** — used everywhere in `manta-shared` and `manta-server` (specifically `crates/manta-server/src/{server,service}/` and `crates/manta-shared/src/{common,manta_backend_dispatcher.rs}`).
- **`anyhow::Error`** — allowed only in `crates/manta-cli/src/cli/` handlers and CLI-only helpers.

The HTTP server converts typed errors to HTTP status codes via `to_handler_error` in `crates/manta-server/src/server/handlers.rs`.

---

## Security model

`manta-server` is a **credential-handling endpoint**: the CLI POSTs Keycloak username/password to `POST /api/v1/auth/token`, and the server proxies them to the configured backend (CSM or OCHAMI) via `service::auth::get_api_token`. The CSM bearer token comes back to the CLI; subsequent authenticated endpoints use it via `Authorization: Bearer`. This is the only path the CLI takes to reach backend credentials — there is no direct CSM/OCHAMI call from `manta-cli`.

This means manta-server is a **single point of compromise** for everyone using it: if it is owned, the attacker gets a chokepoint that sees every auth attempt and holds whatever service-account scoped tokens are configured for the backend. Mitigations split between code and ops:

| Layer | Where | Notes |
|---|---|---|
| Per-source-IP rate limit on `/api/v1/auth/*` | code | `[server].auth_rate_limit_per_minute` (default 60). Implementation in `server::auth_middleware::rate_limit`. |
| Generic 401 on every auth failure | code | `server::handlers::auth_token` returns the same `"invalid credentials"` body regardless of whether the user was unknown or the password was wrong. Detail stays in server-side `tracing::warn!`. |
| Audit event per auth attempt | code | `manta_shared::common::audit::send_auth_audit` emits `{ outcome, username, source_ip, site }` to the configured Kafka producer. Credentials are never logged. |
| Body redaction on `/auth/*` log spans | code | `server::auth_middleware::strip_body_for_logs`. |
| TLS termination, WAF, reverse-proxy rate limit | **ops** | First line of defence; manta-server's in-process limiter is belt-and-braces. |
| Service-account scoping at CSM / Vault | **ops** | Limit what the manta-server-issued tokens can do at the backend. |
| Network segmentation | **ops** | Treat manta-server as a privileged host. |

**Deferred:** forwarding the original client IP to Keycloak via `X-Forwarded-For` on the upstream auth call. The current `AuthenticationTrait::get_api_token` signature in `manta-backend-dispatcher` does not take a header argument, so this would require a sibling-repo upgrade (csm-rs + ochami-rs). Tracked as a follow-up.

---

## Key external dependencies

| Crate | Role |
|-------|------|
| `csm-rs` | HTTP client for HPE Cray System Management APIs: CFS, BOS, HSM, IMS, PCS |
| `ochami-rs` | HTTP client for OpenCHAMI APIs: BSS, SMD |
| `manta-backend-dispatcher` | Trait definitions, shared types, shared error enum |
| `axum` + `axum-server` | HTTPS server with TLS via rustls |
| `utoipa` + `utoipa-swagger-ui` | OpenAPI 3.0 spec generation and Swagger UI serving |
| `clap` | CLI argument parsing |
| `tokio` | Async runtime |
| `minijinja` | Jinja2 template rendering for SAT file processing |
| `rdkafka` | Kafka producer for operation audit trail |
| `git2` | Local git repository operations (repo validation, CFS layer source) |
| `config` | TOML config file loading with environment variable overrides |
| `dialoguer` | Interactive terminal prompts (confirmations, selection lists) |
| `comfy-table` | Terminal table output |
| `reqwest` | HTTP client used by csm-rs and ochami-rs |

---

## SOCKS5 proxy

The SOCKS5 proxy URL is read from `site.socks5_proxy` in `config.toml` and threaded explicitly through the entire call stack — there is no global environment variable or implicit state.

The propagation path:

```
config.toml  socks5_proxy
  └─ main.rs             reads site_details_value.socks5_proxy.as_deref()
       ├─ StaticBackendDispatcher::new(…, socks5_proxy)
       │    ├─ Csm::new(…, socks5_proxy)   — stored as Option<String>
       │    └─ Ochami::new(…, socks5_proxy) — stored as Option<String>
       │         (every http_client call inside csm-rs/ochami-rs receives socks5_proxy
       │          via the Csm/Ochami struct and builds reqwest::Proxy from it)
       ├─ InfraContext { socks5_proxy, … }  — borrowed for the request lifetime
       │    (service functions that call csm-rs directly — e.g. get_node_details —
       │     pass infra.socks5_proxy to the http_client function)
       └─ ServerState { socks5_proxy: Option<String>, … }
            (service functions called from server handlers receive socks5_proxy
             via InfraContext, built from the matching SiteBackend in ServerState)
```

Every function in csm-rs and ochami-rs that builds a `reqwest::Client` accepts `socks5_proxy: Option<&str>` as an explicit parameter placed immediately after `root_cert: &[u8]`. The client is built as:

```rust
let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
};
```

## Audit trail

All mutating CLI operations can emit a Kafka message. Configuration lives under `[auditor]` in `config.toml`. The producer is a lazily-initialised `FutureProducer` in a `OnceLock`; messages are fire-and-forget with a 5-second timeout. Audit calls are made in the service layer via `common::kafka`.

## Hooks

`config.toml` supports pre/post hooks — arbitrary shell commands run before or after certain operations (e.g. `apply sat-file --pre-hook`, `--post-hook`). `common::hooks::run_hook` executes them via a subshell and returns `Error::HookError` if the exit code is non-zero.

---

## Adding a new command

1. Create `crates/manta-cli/src/cli/commands/<verb_noun>.rs` with the clap `Args` struct and an `exec` function.
2. Register it in `crates/manta-cli/src/cli/commands/mod.rs` and add the subcommand variant to the appropriate clap enum in `crates/manta-cli/src/cli/build.rs`.
3. Add the dispatch arm in `crates/manta-cli/src/cli/process/`.
4. If the operation is non-trivial, implement the business logic as a public function in the appropriate `crates/manta-server/src/service/<module>.rs`.
5. If the operation needs a new backend call, add the method to the relevant trait in `manta-backend-dispatcher`, implement it in both `csm-rs` and `ochami-rs`, and add the dispatch arm in `crates/manta-shared/src/backend_dispatcher/`.
6. If the command should also be reachable via the HTTP API, add a handler in `crates/manta-server/src/server/handlers.rs` (with a `#[utoipa::path(...)]` annotation), register the route in `crates/manta-server/src/server/routes.rs`, and add the path and any new schema types to the `#[openapi(...)]` derive in `crates/manta-server/src/server/api_doc.rs`.
