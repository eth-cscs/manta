# Follow-up issue draft: differentiated CLI exit codes

> Draft for a GitHub issue against `eth-cscs/manta`. Spun out of #64
> (error codes) — see `ERROR-CODES-DESIGN.md`.

## Problem

`manta` exits `0` on success and `1` on **any** failure
(`crates/manta-cli/src/main.rs` — single `eprintln!` + `process::exit(1)`
path). Scripts and CI pipelines cannot branch on the failure class (auth
expired vs. resource not found vs. server unreachable) without parsing stderr
text, which is exactly what the #64 error codes were meant to make
unnecessary — but the codes only appear in the message today, not in the exit
status.

This was deliberately kept **out** of #64: changing exit codes is a
user-visible behavioural change for anything that checks `$? == 1`, and
deserves its own decision.

## Proposal

Map error-code *classes* (not individual codes — there are ~30 and exit codes
are a byte) to a small, documented set of exit codes, in the spirit of
`curl`/`ssh`/`grep`:

| Exit code | Class |
|---|---|
| 0 | success |
| 1 | generic / internal failure (unchanged catch-all) |
| 2 | usage / invalid input (clap already uses 2 for parse errors — align) |
| 3 | authentication failure (`MANTA_AUTH_*`, `MANTA_JWT_*`) |
| 4 | resource not found (`MANTA_*_NOT_FOUND`) |
| 5 | conflict / precondition (`MANTA_CONFLICT`, `MANTA_CONFIGURATION_ALREADY_EXISTS`, `MANTA_INSUFFICIENT_RESOURCES`) |
| 6 | connectivity: CLI ↔ manta-server (`MANTA_SERVER_*`, `MANTA_TRANSPORT_ERROR`) |
| 7 | connectivity: manta-server ↔ backend (`MANTA_BACKEND_*`, `MANTA_NETWORK_ERROR`) |

(Exact table to be settled in the issue discussion; the class → exit-code map
lives next to the `ErrorCode` catalog so the two cannot drift.)

The mapping requires the failure's `ErrorCode` to survive to `main.rs` — i.e.
threading a typed error (or an anyhow context marker) through the CLI's
`anyhow` chain instead of collapsing to a string. That plumbing is the bulk of
the work.

## Tasks

- [ ] Settle the class → exit-code table
- [ ] Thread `ErrorCode` through the CLI error path to `main.rs`
- [ ] Document in `CLI.md` + `ERRORS.md`; note the compatibility change in the changelog
- [ ] Tests: one integration test per exit-code class

## Acceptance criteria

- `manta` exits with the documented code per failure class; `1` remains the
  catch-all, so existing `!= 0` checks keep working.
- Documented as a breaking-ish change in release notes.

## Dependencies

- Requires #64 (the `ErrorCode` catalog) to be merged first.
