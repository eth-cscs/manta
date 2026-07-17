# Follow-up issue draft: audit HTTP status mapping + OpenAPI error responses

> Draft for a GitHub issue against `eth-cscs/manta`. Spun out of #64
> (error codes) — see `ERROR-CODES-DESIGN.md`. Two small pre-existing
> inconsistencies were found while cataloguing errors for #64; both were left
> untouched there to avoid bundling behaviour changes.

## Problem

1. **`MissingField` maps to HTTP 500.** In `to_handler_error`
   (`crates/manta-server/src/server/handlers/mod.rs`), the
   `MissingField` variant falls through to the 500 catch-all, although some of
   its uses read like client-side 4xx conditions (and others like 502-ish
   "backend data incomplete"). The variant's doc comment and the mapping
   disagree in spirit. Worth deciding case by case whether it should be 400,
   422, 502, or stay 500 — possibly splitting the variant.

2. **Per-endpoint OpenAPI error responses are hand-written and can drift.**
   Every handler declares its own `(status = N, body = ErrorResponse)` utoipa
   `responses(...)` list, independent of the actual runtime ladder in
   `to_handler_error`. Nothing checks they agree, so the published
   `openapi.json` can advertise statuses an endpoint never returns (and miss
   ones it does). With #64 the divergence becomes more visible, since
   `ERRORS.md` now documents the real mapping.

## Proposal

- Decide and fix the `MissingField` status (an append-only `ErrorCode` split —
  e.g. `MANTA_MISSING_FIELD_INPUT` vs keeping `MANTA_MISSING_FIELD` — is
  allowed by the stability policy if the variant splits).
- Reduce the utoipa drift surface: either generate the per-endpoint error
  `responses(...)` from a shared helper/macro fed by the same table
  `to_handler_error` uses, or add a test that walks the OpenAPI spec and
  asserts every declared error status is producible (and vice versa for the
  common ones).

## Tasks

- [ ] Classify all `MissingField` raise sites; pick target statuses
- [ ] Implement shared error-`responses` helper or spec-vs-ladder test
- [ ] Regenerate `openapi.json`; update `ERRORS.md` if statuses change

## Acceptance criteria

- No endpoint advertises an error status it cannot return (for the shared
  ladder statuses).
- `MissingField` status is a deliberate decision, documented in `ERRORS.md`,
  not a fall-through accident.

## Dependencies

- Requires #64 to be merged first (uses its catalog + `ERRORS.md`).
