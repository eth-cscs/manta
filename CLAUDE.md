# Error handling convention

## Two-tier error type rule

This project uses two error types:

- **`manta_backend_dispatcher::error::Error`** — structured error type for shared backend-touching code: everything under `crates/manta-server/src/server/`, `crates/manta-server/src/service/`, `crates/manta-server/src/backend_dispatcher/`, and `crates/manta-server/src/dispatcher.rs`.
- **`anyhow::Error`** — allowed only in `crates/manta-cli/src/` (handlers and CLI-only command functions). CLI handlers terminate with `eprintln!` + `process::exit()`, so anyhow's ergonomics are appropriate there.

`manta_shared::common::error::MantaError` is the third type. It's used by:
- `manta-shared`'s remaining helpers (`common::config` loader)
- `manta-server`'s internal helpers under `server/common/` (`audit`, `jwt_ops`, `kafka`)
- `manta-cli`'s SAT-file Jinja2 renderer (`dispatch/apply/sat_file/render.rs`)

Server code maps `MantaError` to `BackendError` at call sites via `crates/manta-server/src/wire_conv.rs::to_backend`.

## The boundary rule

Handlers under `crates/manta-server/src/server/handlers/` **must only call functions in `crates/manta-server/src/service/`** (which includes `service/infra_backend.rs`'s `InfraContext` methods) or shared helpers in `manta-shared`, never CLI functions. The service layer uses `manta_backend_dispatcher::error::Error` throughout; handlers convert these to HTTP responses via `to_handler_error`.

Functions called exclusively from CLI entrypoints may use `anyhow::Error`.

## Enforcement

A CI step in `.github/workflows/ci.yml` greps for `use anyhow` in `crates/manta-server/src/{server,service,backend_dispatcher,dispatcher.rs}` and `crates/manta-shared/src/common/` and fails the build if found.

# Workspace layout

manta is a Cargo workspace:

```
crates/manta-shared/   (lib)  — wire types, common helpers, backend dispatcher
crates/manta-cli/      (bin)  — terminal client
crates/manta-server/   (bin)  — Axum HTTPS server
```

Build a single crate with `cargo build -p manta-cli` or `cargo build -p manta-server`. The two binaries don't depend on each other; both depend on `manta-shared`.
