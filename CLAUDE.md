# Error handling convention

## Two-tier error type rule

This project uses two error types:

- **`manta_backend_dispatcher::error::Error`** — structured error type for all shared code: `src/server/`, `src/service/`, `src/common/`, `src/manta_backend_dispatcher.rs`, and any `src/cli/` module callable from the HTTP server.
- **`anyhow::Error`** — allowed only in `src/cli/` (handlers and CLI-only command functions). CLI handlers terminate with `eprintln!` + `process::exit()`, so anyhow's ergonomics are appropriate there.

## The boundary rule

Any function in `src/cli/` that is called from `src/server/handlers.rs` **must** return `manta_backend_dispatcher::error::Error`, not `anyhow::Error`. The HTTP server handler converts these via `to_handler_error`.

Functions called exclusively from CLI entrypoints may use `anyhow::Error`.

## Enforcement

A CI step in `.github/workflows/ci.yml` greps for `use anyhow` in `src/server/`, `src/service/`, `src/common/`, and `src/manta_backend_dispatcher.rs` and fails the build if found.
