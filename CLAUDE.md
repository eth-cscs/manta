# Error handling convention

## Two-tier error type rule

This project uses two error types:

- **`manta_backend_dispatcher::error::Error`** — structured error type for all shared code: everything under `crates/manta-server/src/server/`, `crates/manta-server/src/service/`, `crates/manta-shared/src/common/`, and `crates/manta-shared/src/manta_backend_dispatcher.rs`.
- **`anyhow::Error`** — allowed only in `crates/manta-cli/src/cli/` (handlers and CLI-only command functions). CLI handlers terminate with `eprintln!` + `process::exit()`, so anyhow's ergonomics are appropriate there.

## The boundary rule

Handlers under `crates/manta-server/src/server/handlers/` **must only call functions in `crates/manta-server/src/service/`** (or shared helpers in `manta-shared`), never CLI functions. The service layer uses `manta_backend_dispatcher::error::Error` throughout; handlers convert these to HTTP responses via `to_handler_error`.

`manta-shared`'s pure helpers (audit, jwt_ops, kafka, config loader, sat-file Jinja2 renderer, network probe) return `manta_shared::common::error::MantaError`; server code maps these to `BackendError` at call sites via `crates/manta-server/src/wire_conv.rs::to_backend`.

Functions called exclusively from CLI entrypoints may use `anyhow::Error`.

## Enforcement

A CI step in `.github/workflows/ci.yml` greps for `use anyhow` in `crates/manta-server/src/{server,service,backend_dispatcher,manta_backend_dispatcher.rs}` and `crates/manta-shared/src/common/` and fails the build if found.

# Workspace layout

manta is a Cargo workspace:

```
crates/manta-shared/   (lib)  — wire types, common helpers, backend dispatcher
crates/manta-cli/      (bin)  — terminal client
crates/manta-server/   (bin)  — Axum HTTPS server
```

Build a single crate with `cargo build -p manta-cli` or `cargo build -p manta-server`. The two binaries don't depend on each other; both depend on `manta-shared`.
