# Error codes — design for issue #64

- **Status:** accepted (2026-07-17), implementation in progress on `feature/custom-error-codes`
- **Issue:** https://github.com/eth-cscs/manta/issues/64
- **Follow-ups split out:** see `FOLLOWUP-upstream-error-granularity.md`,
  `FOLLOWUP-cli-exit-codes.md`, `FOLLOWUP-error-status-and-openapi-audit.md`

This document records how issue #64 ("Implement Custom Error Handling with
Error Codes and Messages") is being implemented, and where — and why — the
implementation deliberately deviates from the issue text. The issue predates
the current architecture (it speaks of error *classes* extending a base class,
and of integrating codes into the csm/ochami libraries directly); the decisions
below map its intent onto what the codebase actually is today.

## Context

Manta already has a consistent typed-error structure (see
`ARCHITECTURE.md` § "Error handling conventions"): a three-type partition
(`BackendError` in the server's service layer, `MantaError` in `manta-shared`,
`anyhow` in the CLI), CI-enforced, with two chokepoints every error flows
through:

- server → wire: `to_handler_error` in
  `crates/manta-server/src/server/handlers/mod.rs`, producing the JSON body
  `ErrorResponse { error: String }` plus an HTTP status;
- wire → user: `OpenApiResultExt::into_anyhow` in
  `crates/manta-cli/src/http_client/client.rs`, which extracts the body and
  renders it on stderr.

What is missing — and what this work adds — is a **stable, unique,
documented code per failure class**, carried on the wire, in server logs, and
in CLI output. No error codes of any kind existed before this change.

## Decision 1 — symbolic codes, not numeric

Codes are SCREAMING_SNAKE_CASE identifiers: `MANTA_INVALID_PATTERN`,
`MANTA_BACKEND_TIMEOUT`, `MANTA_SESSION_NOT_FOUND`. Not `MANTA_001`.

Rationale:

- **Self-describing.** A user pasting `MANTA_BACKEND_TIMEOUT` into a support
  channel has communicated most of the story before anyone opens the docs;
  `MANTA_017` communicates nothing without a lookup table.
- **No allocation bookkeeping.** Numeric codes need a registry discipline
  ("next free number"), invite collisions between concurrent changes, and —
  if ranges encode a category, e.g. `MANTA_4xx` = upstream — bake a
  *classification* into an *identifier*. Classifications get revised;
  identifiers must never change. (Live example: `MissingField` currently maps
  to HTTP 500 but reads like a 4xx — under range-encoded numbering, fixing
  that classification would force a renumbering, which the stability policy
  forbids.)
- **Mechanical mapping.** Symbolic codes map ~1:1 onto the existing error-enum
  variants, so the mapping layer is trivial to review and test.
- Equally greppable, stable, and machine-parseable as numbers.

Underscores (not hyphens) because SCREAMING_SNAKE_CASE is the Rust convention
for constants: the code string is byte-identical to a valid Rust identifier,
double-click-selects as one token, and matches the project's env-var style
(`MANTA_CLI_CONFIG`).

## Decision 2 — a single `MANTA_` namespace (no `CSM_` / `OCHAMI_` prefixes)

The issue suggests per-origin prefixes (`CSM_003`, `OCHAMI_002`). That is not
implementable honestly at the layer where codes are assigned: errors from both
backends arrive through the same foreign `manta_backend_dispatcher::error::Error`
type, whose single `CsmError { status, detail, body }` variant carries backend
HTTP errors regardless of which backend produced them. A per-origin prefix
would be a guess presented as a fact.

Therefore: one `MANTA_*` namespace. The upstream origin, status, and detail are
preserved in the **message**, which accompanies the code everywhere.

## Decision 3 — stability policy (append-only)

Error codes are a public contract:

- Codes are **never renamed, renumbered, reused, or deleted**. A code that
  becomes obsolete is *deprecated* (kept in the catalog, marked as such).
- **Messages are not stable** and may be reworded freely; the code is the key
  to reference in scripts, docs, and support channels.
- Codes are explicit string literals in the catalog — never derived from Rust
  variant names via macros — so a future variant rename cannot silently change
  a published code. Pin tests lock each code string.

This policy is restated in `ERRORS.md` (the user-facing catalog).

## Decision 4 — wire format and version skew

`ErrorResponse` gains one field:

```json
{ "error": "session foo not found", "code": "MANTA_SESSION_NOT_FOUND" }
```

`code` is **optional** on the wire. A new CLI talking to an old server (no
`code` in the body) and an old CLI talking to a new server (unknown extra
field, ignored by serde) both degrade gracefully to today's behaviour. The
OpenAPI schema change flows into the progenitor-generated CLI client via the
checked-in `openapi.json` (regenerated as part of this change; CI gates it).

CLI rendering: `HTTP 404 [MANTA_SESSION_NOT_FOUND]: session foo not found`.
CLI-local failures (transport, config) use the same `[CODE] message` shape.
Server logs carry the code in the existing `to_handler_error` tracing lines.

## Decision 5 — granularity: one code per failure class

One code per failure *class* (`MANTA_NOT_FOUND` is one code wherever a generic
not-found occurs), not per call site. This is the only maintainable choice and
matches the enum structure; the initial catalog is ~30 codes.

Honest limitation: the catalog can only be as granular as the variants it maps
from. `BackendError` is a pinned crates.io crate
(`manta-backend-dispatcher 1.0.0-beta.13`), so all deep CSM/OCHAMI failures
collapse into a small `MANTA_BACKEND_*` family, with the detail only in the
message. Finer granularity requires upstream variant work — split out as a
follow-up (`FOLLOWUP-upstream-error-granularity.md`), not blocking this issue.

## Implementation shape

- **Catalog:** `crates/manta-shared/src/common/error_code.rs` — an `ErrorCode`
  enum with `as_str()` (the wire string), a one-line description per code, and
  a `const ALL` table for iteration/tests. Lives in `manta-shared` because both
  binaries need it and it keeps the CI layering rules intact.
- **Mappings at the chokepoints, not at raise sites:**
  - `MantaError::error_code()` (exhaustive, in `manta-shared`);
  - a server-side `&BackendError → ErrorCode` function next to
    `wire_conv::to_backend` (exhaustive match over the foreign type — adding an
    upstream variant breaks the build, which is the point);
  - `to_handler_error` stamps the code into the body and the log line; the few
    helpers that construct `ErrorResponse` directly (extractors,
    `serialize_or_500`, `require_url`, `parse_iso_datetime`, …) pass their own;
  - CLI: `into_anyhow` / `unwrap_error_body` read the wire code;
    `categorise_transport_error` and the CLI marker errors get catalog codes.
- **Docs:** `ERRORS.md` — full code → meaning → HTTP status → remediation
  table plus the stability policy; linked from the user docs.
- **Tests:** catalog uniqueness + format; per-variant mapping pins (same style
  as the existing `wire_conv` tests); handler-boundary body checks; CLI
  extraction; a sync test asserting every catalog code appears in `ERRORS.md`.

## Provisional catalog (illustrative, finalised in `ERRORS.md`)

| Code | Typical HTTP | Meaning |
|---|---|---|
| `MANTA_BAD_REQUEST` | 400 | Request rejected by validation |
| `MANTA_INVALID_PATTERN` | 400 | User-supplied pattern/hostlist/glob didn't parse |
| `MANTA_INVALID_NODE_ID` | 400 | Node id (xname) invalid |
| `MANTA_INVALID_DATETIME` | 400 | Datetime filter didn't parse |
| `MANTA_UNSUPPORTED_BACKEND` | 400 | Operation not supported by this backend type |
| `MANTA_AUTH_TOKEN_NOT_FOUND` | 401 | No/expired auth token for the site |
| `MANTA_JWT_MALFORMED` | 401 | Token structurally invalid |
| `MANTA_NOT_FOUND` | 404 | Generic resource lookup failed |
| `MANTA_SESSION_NOT_FOUND` | 404 | CFS session not found |
| `MANTA_CONFIGURATION_NOT_FOUND` | 404 | CFS configuration not found |
| `MANTA_SITE_NOT_FOUND` | 404 | `X-Manta-Site` names a site the server doesn't host |
| `MANTA_CONFLICT` | 409 | Resource state conflict |
| `MANTA_CONFIGURATION_ALREADY_EXISTS` | 409 | Configuration name taken |
| `MANTA_INSUFFICIENT_RESOURCES` | 422 | Not enough capacity to satisfy request |
| `MANTA_NOT_CONFIGURED` | 501 | Optional server feature (vault, k8s) not configured |
| `MANTA_BACKEND_HTTP_ERROR` | upstream status | Backend (CSM/OCHAMI) returned an HTTP error |
| `MANTA_BACKEND_TIMEOUT` | 504 | manta-server → backend call timed out |
| `MANTA_BACKEND_CONNECT_FAILED` | 502/500 | manta-server could not connect to the backend |
| `MANTA_NETWORK_ERROR` | 500 | Other outbound HTTP failure |
| `MANTA_MISSING_FIELD` | 500 | Required field absent in backend data |
| `MANTA_IO_ERROR` / `MANTA_SERDE_ERROR` / `MANTA_YAML_ERROR` / `MANTA_TOML_ERROR` / `MANTA_CONFIG_ERROR` | 500 | Internal I/O / (de)serialisation / config failures |
| `MANTA_KAFKA_ERROR` / `MANTA_K8S_ERROR` / `MANTA_CONSOLE_ERROR` / `MANTA_TEMPLATE_ERROR` / `MANTA_HOOK_ERROR` / `MANTA_GIT_ERROR` | 500 | Subsystem failures |
| `MANTA_INTERNAL` | 500 | Catch-all (`Message`/`Other`) |
| `MANTA_SERVER_UNREACHABLE` | — (CLI) | CLI could not connect to manta-server |
| `MANTA_SERVER_TIMEOUT` | — (CLI) | CLI-side request timeout talking to manta-server |
| `MANTA_TRANSPORT_ERROR` | — (CLI) | Other CLI ↔ manta-server transport failure |

## Non-goals (this issue)

- **Differentiated CLI exit codes** — behavioural change for scripts; see
  `FOLLOWUP-cli-exit-codes.md`. Exit code stays `1` on any failure.
- **Codes inside csm-rs / ochami-rs / manta-backend-dispatcher** — separate
  pinned repos; see `FOLLOWUP-upstream-error-granularity.md`.
- **Fixing the `MissingField` → 500 status and the hand-written per-endpoint
  utoipa `responses(...)` drift** — pre-existing; see
  `FOLLOWUP-error-status-and-openapi-audit.md`.
