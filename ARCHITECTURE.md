# Manta Architecture

This document describes the internal structure of the manta codebase for contributors.

---

## Layer overview

```
User
  │
  ├─ CLI mode
  │    src/cli/          — argument parsing, output tables, user prompts
  │      └─ src/service/ — business logic, orchestration, filtering
  │           └─ src/backend_dispatcher/   — trait dispatch to backend
  │                └─ csm-rs / ochami-rs   — HTTP clients for CSM / OpenCHAMI
  │
  └─ HTTP server mode
       src/server/       — axum HTTPS server, JWT auth middleware
         └─ src/service/ — same service layer as CLI
              └─ src/backend_dispatcher/
                   └─ csm-rs / ochami-rs
```

Both modes share the `service` and `backend_dispatcher` layers. The CLI and server layers are the only parts that differ.

---

## Entry point: `src/main.rs`

Startup runs in two phases:

1. **Single-threaded phase** — parse CLI args, load `~/.config/manta/config.toml`. The SOCKS5 proxy URL is read from `site.socks5_proxy` in the config.
2. **Multi-threaded phase** — start the tokio runtime, construct a `StaticBackendDispatcher` (passing `socks5_proxy`), then branch:
   - `manta serve ...` → start the HTTPS server
   - anything else → run the CLI command

---

## Layer responsibilities

### `src/cli/`

Presentation layer. Responsibilities:

- **`build/`** — Clap command and subcommand definitions.
- **`process/`** — Argument extraction and dispatch to the service layer.
- Output formatting via `comfy-table` for terminal tables.
- Interactive prompts via `dialoguer`.
- Error handling via `anyhow::Error`; CLI handlers terminate with `eprintln!` + `process::exit()`.

CLI code **must not** contain business logic. It calls service functions with typed parameters and formats their results.

### `src/service/`

Business logic layer. Modules: `session`, `configuration`, `group`, `node`, `image`, `template`, `boot_parameters`, `kernel_parameters`, `hardware`, `cluster`, `sat_file`, `migrate`, `redfish_endpoints`.

Each module receives an `&InfraContext<'_>` plus a bearer token and typed parameters, and returns typed results. This layer:

- Orchestrates multi-step operations (e.g. create config → build image → create session).
- Applies filtering, sorting, and business rules.
- Uses `manta_backend_dispatcher::error::Error` (not `anyhow`).
- Has no knowledge of terminal output or HTTP request/response shapes.

### `src/backend_dispatcher/`

Trait implementation glue. Implements all `manta-backend-dispatcher` traits (`CfsTrait`, `GroupTrait`, `BootParametersTrait`, etc.) on `StaticBackendDispatcher` using a `dispatch!` macro. The macro expands to a `match` that routes each method call to either the `Csm` or `OCHAMI` variant.

### `src/manta_backend_dispatcher.rs`

Defines the `StaticBackendDispatcher` enum:

```rust
pub enum StaticBackendDispatcher {
    CSM(Csm),
    OCHAMI(Ochami),
}
```

`StaticBackendDispatcher::new(backend_type, base_url, root_cert, socks5_proxy)` reads the `backend` field from the site config and constructs the appropriate variant, forwarding `socks5_proxy` to both `Csm::new` and `Ochami::new`.

### `src/common/`

Shared utilities used by both CLI and server:

| Module | Purpose |
|--------|---------|
| `config/` | Load and validate `~/.config/manta/config.toml` |
| `jwt_ops` | Decode and validate JWT bearer tokens |
| `kafka` | Fire-and-forget audit event producer |
| `hooks` | Execute pre/post shell hooks |
| `vault` | Fetch secrets from HashiCorp Vault |
| `log_ops` | Logger initialisation |

### `src/server/`

Axum HTTPS server. Key files:

| File | Purpose |
|------|---------|
| `mod.rs` | `start_server` — binds TLS, builds router, starts accepting connections |
| `routes.rs` | Registers ~40 REST endpoints + 2 WebSocket upgrades under `/api/v1/` |
| `handlers.rs` | Per-endpoint functions: extract bearer token, deserialise params, call service, serialise response |
| `middleware.rs` | JWT authentication middleware applied to all routes |

`ServerState` (wrapped in `Arc`) owns all infrastructure: backend dispatcher, TLS certificates, optional Vault/k8s URLs.

---

## Context objects

| Type | Used by | Contents |
|------|---------|---------|
| `InfraContext<'_>` | Service layer | Backend dispatcher, base URLs, root CA cert, SOCKS5 proxy, optional vault/k8s URLs |
| `AppContext` | CLI layer | Composes `InfraContext` + `CliConfig` (active HSM group, Kafka audit config, raw settings) |
| `Arc<ServerState>` | HTTP server | Infrastructure behind a reference-counted pointer; each handler calls `.infra_context()` |

---

## Backend selection

The config file (`~/.config/manta/config.toml`) has one or more site sections:

```toml
site = "cscs_prod"          # active site

[sites.cscs_prod]
backend = "csm"             # or "ochami"
shasta_base_url = "https://api.cscs.ch"
root_ca_cert_file = "cscs_root_cert.pem"

[sites.local_ochami]
backend = "ochami"
shasta_base_url = "https://foobar.openchami.cluster:8443"
root_ca_cert_file = "ochami_root_cert.pem"
```

The active site is chosen by `site = "<name>"` at the top level, overridable per-invocation with `--site <name>`. `StaticBackendDispatcher::new` reads the `backend` string and constructs `CSM(...)` or `OCHAMI(...)`.

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

- **`manta_backend_dispatcher::error::Error`** — used in `src/server/`, `src/service/`, `src/common/`, `src/manta_backend_dispatcher.rs`, and any `src/cli/` function callable from the HTTP server.
- **`anyhow::Error`** — allowed only in `src/cli/` handlers and CLI-only functions that never run in server mode.

The HTTP server converts typed errors to HTTP status codes via `to_handler_error` in `src/server/handlers.rs`.

---

## Key external dependencies

| Crate | Role |
|-------|------|
| `csm-rs` | HTTP client for HPE Cray System Management APIs: CFS, BOS, HSM, IMS, PCS |
| `ochami-rs` | HTTP client for OpenCHAMI APIs: BSS, SMD |
| `manta-backend-dispatcher` | Trait definitions, shared types, shared error enum |
| `axum` + `axum-server` | HTTPS server with TLS via rustls |
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
            (server handlers that call CLI commands directly — e.g. apply_ephemeral_env —
             pass state.socks5_proxy.as_deref())
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

1. Create `src/cli/commands/<verb_noun>.rs` with the clap `Args` struct and an `exec` function.
2. Register it in `src/cli/commands/mod.rs` and add the subcommand variant to the appropriate clap enum in `src/cli/build.rs`.
3. Add the dispatch arm in `src/cli/process/`.
4. If the operation is non-trivial, implement the business logic as a public function in the appropriate `src/service/<module>.rs`.
5. If the operation needs a new backend call, add the method to the relevant trait in `manta-backend-dispatcher`, implement it in both `csm-rs` and `ochami-rs`, and add the dispatch arm in `src/backend_dispatcher/`.
6. If the command should also be reachable via the HTTP API, add a handler in `src/server/handlers.rs` and register the route in `src/server/routes.rs`.
