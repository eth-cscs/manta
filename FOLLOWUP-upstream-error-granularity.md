# Follow-up issue draft: richer error granularity from the backend crates

> Draft for a GitHub issue. Spun out of #64 (error codes) — see
> `ERROR-CODES-DESIGN.md`. File against `eth-cscs/manta` with linked issues in
> `eth-cscs/manta-backend-dispatcher`, `eth-cscs/csm-rs`, and
> `OpenCHAMI/ochami-rs` as the work is scheduled.

## Problem

Issue #64 introduced stable `MANTA_*` error codes, assigned where errors cross
manta's boundaries. The catalog's granularity is capped by the variant list of
`manta_backend_dispatcher::error::Error`, which is a pinned crates.io
dependency:

- A single `CsmError { status, detail, body }` variant carries **every** HTTP
  error from **either** backend, so all deep CSM/OCHAMI failures collapse into
  one `MANTA_BACKEND_HTTP_ERROR` code, with the distinguishing detail only in
  the free-text message.
- The origin backend (CSM vs OCHAMI) is not attributable at manta's mapping
  layer, which is why #64 rejected the originally proposed `CSM_*` / `OCHAMI_*`
  code prefixes.

Users and support therefore get one coarse code for the most interesting class
of failures (backend rejections), which limits the troubleshooting value the
codes were introduced for.

## Proposal

1. In `manta-backend-dispatcher`: split the backend-HTTP-error surface into
   variants that carry (a) the originating backend (CSM / OCHAMI), and
   (b) enough structure to distinguish the common failure classes
   (auth rejected upstream, resource missing upstream, upstream validation
   error, upstream 5xx, …). An `origin` field on the existing variant is the
   minimal version.
2. In `csm-rs` / `ochami-rs`: populate those variants at the call sites that
   currently produce the generic error.
3. In manta: extend the `ErrorCode` catalog (append-only, per the stability
   policy in `ERRORS.md`) with the finer codes, e.g. `MANTA_CSM_AUTH_REJECTED`,
   `MANTA_OCHAMI_NOT_FOUND`, and map the new variants onto them in the
   server-side mapping function (the exhaustive match will force this at
   compile time when the dependency is bumped).

## Tasks

- [ ] Design the variant split in `manta-backend-dispatcher` (issue/PR there)
- [ ] Adopt in `csm-rs` and `ochami-rs`
- [ ] Bump the pinned versions in manta, extend the catalog + mapping + `ERRORS.md`
- [ ] Extend mapping pin tests

## Acceptance criteria

- Backend-originated failures carry a code that identifies at least the origin
  backend and the failure class, not just "a backend HTTP error happened".
- No existing code changes meaning (append-only catalog).

## Dependencies

- Requires #64 to be merged first.
- Requires releases of the three sibling crates; manta pins exact versions.
