# Design — hardware-cluster pin/unpin: migrate from server to CLI

**Date:** 2026-06-22
**Scope:** `crates/manta-server`, `crates/manta-cli`, `crates/manta-shared`. One PR.

## Problem

`comfy-table = "7.2.2"` is a dependency of `manta-server`. It is used in
exactly one place: `crates/manta-server/src/service/hw_cluster/scoring.rs::print_score_table`,
which renders a colored Unicode grid of hardware-component scores and
emits it via `tracing::info!` during pin/unpin iterations. The output
lands in server logs; the CLI never sees it.

A TTY-rendering crate in a service layer is a layering smell — `comfy-table`
suits CLIs. The narrow fix (drop the dep, replace with structured tracing)
would leave the score visibility behind. The broader fix — move the whole
`hw_cluster` orchestration to the CLI where its presentation already
belongs — also resolves a longer-standing architectural pull: the algorithm
is pure compute over data the CLI can fetch directly, and the server only
hosts it for historical reasons.

## Goal

Move the hardware-cluster pin/unpin / add-component / delete-component
functionality from the server's service+handler layers into a new CLI
module, removing `comfy-table` from `manta-server` as a side effect.

The three flows (`manta apply hardware group`, `manta add hardware`,
`manta delete hardware`) keep the same CLI surface (args, output,
exit semantics). Their implementation moves from "thin OpenAPI wrapper"
to "thick local orchestrator over primitive REST endpoints."

## Plan in three steps

1. **CLI fetches target and parent group details from the server**
   (one GET per group, returning per-node `NodeSummary` lists).
2. **CLI calculates scores and computes new group composition** locally —
   the pure-math heart of the existing algorithm.
3. **CLI submits the new member lists; server overwrites the groups**
   via a new `PUT /api/v1/groups/{name}/members` endpoint that takes the
   desired final member list and replaces the group atomically.

Step 3 needs a new REST primitive — no existing endpoint replaces a
group's full member list. `POST` is add-only and `DELETE` is remove-only
on the same path. The new `PUT` wraps the existing
`backend.update_group_members(remove[], add[])` and is the only new
server-side surface this PR introduces.

## New server endpoint: `PUT /api/v1/groups/{name}/members`

### Wire types (added to `crates/manta-shared/src/types/api/group.rs`)

```rust
/// Request body for `PUT /api/v1/groups/{name}/members`.
///
/// Replaces the group's membership with `members`. The server computes
/// the diff against the current state and forwards add/remove sets to
/// the backend in a single atomic call.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReplaceGroupMembersRequest {
    /// Final desired member xnames after the call returns.
    pub members: Vec<String>,
}

/// Response body — names the actual deltas applied so the CLI can
/// confirm what changed without a follow-up GET.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReplaceGroupMembersResponse {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub final_members: Vec<String>,
}
```

### Handler (`crates/manta-server/src/server/handlers/group.rs`)

```rust
#[utoipa::path(put, path = "/groups/{name}/members", tag = "groups",
  params(("name" = String, Path, description = "Group name"), SiteHeader),
  request_body = ReplaceGroupMembersRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Members replaced", body = ReplaceGroupMembersResponse),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn replace_group_members(
    ctx: RequestCtx,
    Path(name): Path<String>,
    Json(body): Json<ReplaceGroupMembersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let infra = ctx.infra();

    service::authorization::validate_user_group_access(&infra, &ctx.token, &name)
        .await.map_err(to_handler_error)?;

    let response = service::group::replace_group_members(
        &infra, &ctx.token, &name, body.members,
    ).await.map_err(to_handler_error)?;

    Ok(Json(response))
}
```

### Service function (`crates/manta-server/src/service/group.rs`)

Reads current members, computes the diff, validates per-xname access on
the union of touched xnames, calls `backend.update_group_members`, and
returns the actual deltas.

```rust
pub async fn replace_group_members(
    infra: &InfraContext<'_>,
    token: &str,
    name: &str,
    desired_members: Vec<String>,
) -> Result<ReplaceGroupMembersResponse, Error> {
    let current = infra.backend.get_group(token, name).await?
        .members.unwrap_or_default().ids.unwrap_or_default();

    let current_set: HashSet<&str> = current.iter().map(String::as_str).collect();
    let desired_set: HashSet<&str> = desired_members.iter().map(String::as_str).collect();

    let add: Vec<String> = desired_set.difference(&current_set)
        .map(|s| s.to_string()).collect();
    let remove: Vec<String> = current_set.difference(&desired_set)
        .map(|s| s.to_string()).collect();

    let mut touched: Vec<String> = add.iter().chain(remove.iter()).cloned().collect();
    if !touched.is_empty() {
        validate_user_group_members_access(infra, token, &touched).await?;
    }

    let remove_ref: Vec<&str> = remove.iter().map(String::as_str).collect();
    let add_ref: Vec<&str> = add.iter().map(String::as_str).collect();
    infra.backend.update_group_members(token, name, &remove_ref, &add_ref).await?;

    touched.sort();
    let mut final_members = desired_members;
    final_members.sort();
    Ok(ReplaceGroupMembersResponse {
        added: add, removed: remove, final_members,
    })
}
```

Authorization shape mirrors today's `POST /groups/{name}/members`:
per-group on `name`, per-xname on the touched set. No change to
`POST`/`DELETE` semantics on the same path — they keep their add-only /
remove-only behaviour as today.

## CLI module layout

New module at `crates/manta-cli/src/hw_cluster/` mirrors the deleted
server-side layout so the diff is reviewable file-for-file:

```
crates/manta-cli/src/hw_cluster/
├── mod.rs              # HwClusterMode enum (local), result types,
│                       #   MEMORY_CAPACITY_LCM constant, public re-exports
├── scoring.rs          # Pure-math: scarcity scores, node scoring,
│                       #   candidate selection, parse_hw_pattern,
│                       #   print_score_table (writes to stderr)
├── pin_unpin.rs        # Pin/Unpin selection algorithms,
│                       #   parse_hw_pattern_usize, validate_resource_sufficiency,
│                       #   apply_group_updates (REST orchestrator using PUT)
├── apply.rs            # Coordinators: apply_hw_configuration,
│                       #   add_hw_component, delete_hw_component
│                       #   — take &MantaClient instead of &InfraContext
└── tests.rs            # 1172 lines of pure-math tests, ported verbatim
                        #   (only the use super::scoring::{…} paths change)
```

`HwClusterMode` is redefined locally in `mod.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HwClusterMode {
    #[default]
    Pin,
    Unpin,
}
```

No `Serialize`/`Deserialize`/`ToSchema` derives — the enum no longer
crosses the wire. Its prior cross-crate home in `manta-shared` is deleted.

Error type throughout the new module is `anyhow::Error`, per the CLAUDE.md
two-tier convention. Existing `Error::InvalidPattern(...)`,
`Error::InsufficientResources(...)`, `Error::NotFound(...)` construction
sites become `anyhow::bail!("…")` with the same message strings preserved.

## Per-command data flow

### `manta apply hardware group --pattern <p> [--create-target-group] [--delete-empty-parent-group] [--dry-run]`

```
1. parse_hw_pattern_usize(pattern)                                  (local)
2. GET  /groups/hardware?hsm_group=<target>     → Vec<NodeSummary>
3. GET  /groups/hardware?hsm_group=<parent>     → Vec<NodeSummary>
4. NodeSummary → per-node component counts                          (local, ~30 LOC)
5. validate_resource_sufficiency(...)                               (local)
6. ensure_target_group_exists:
     IF missing AND --create-target-group AND NOT dryrun:
       POST /groups   { label: <target>, ... }
7. calculate scores; iterate Pin/Unpin loop                         (local)
     per iteration: print_score_table → stderr
8. IF NOT dryrun:
     PUT /groups/<target>/members  { members: <new_target_xnames> }
     PUT /groups/<parent>/members  { members: <new_parent_xnames> }
     IF parent_will_be_empty AND --delete-empty-parent-group:
       DELETE /groups/<parent>
9. render result via output/action_result.rs                        (existing path)
```

**Call count:** 4 minimum (2 reads + 2 writes), 6 worst case
(+ POST create, + DELETE parent).

### `manta add hardware`

```
1. ensure_target_group_exists                                      (POST if needed)
2. parse_hw_pattern                                                (local)
3. GET  /groups/<target>                       → current target members
4. GET  /groups/hardware?hsm_group=<parent>    → Vec<NodeSummary>
                                                  (also yields current parent members)
5. compute_final_parent_summary, scarcity_scores                   (local)
6. calculate_target_group_unpin(...)                               (local)
7. IF NOT dryrun:
     PUT /groups/<target>/members  { members: <existing_target ∪ nodes_to_move> }
     PUT /groups/<parent>/members  { members: <existing_parent \ nodes_to_move> }
8. render result
```

**Call count:** 4 minimum, 5 worst case (+ POST create).
Today this flow is 1 read + 2N writes (one DELETE + one POST per moved xname);
the migration collapses writes to 2 PUTs regardless of N, at the cost of
one extra read.

(The `GET /groups/<target>` is necessary because the new target member
list is `existing + moved` — `apply` doesn't need this read because its
`GET /groups/hardware?hsm_group=<target>` already yields the xname set
in the `NodeSummary` list.)

### `manta delete hardware`

```
1. GET  /groups/<target>                       → existence + member list
2. parse_hw_pattern                                                (local)
3. GET  /groups/hardware?hsm_group=<target>    → Vec<NodeSummary>
4. handle_empty_target short-circuit if no members
5. GET  /groups/hardware?hsm_group=<parent>    → Vec<NodeSummary>
6. compute_delete_final_summary, scarcity_scores                   (local)
7. calculate_target_group_unpin(...)                               (local)
8. IF NOT dryrun:
     PUT /groups/<target>/members  { members: <new_target_xnames> }
     PUT /groups/<parent>/members  { members: <new_parent_xnames> }
     IF target_will_be_empty AND delete_group flag set:
       DELETE /groups/<target>
9. render result
```

**Call count:** 5 minimum (3 reads + 2 writes), 6 with optional DELETE.

The exact CLI flag name (`--delete-hsm-group`, `--delete-target-group`,
etc.) is preserved from today's dispatcher and is not respecified here.

## What gets deleted

- `crates/manta-server/src/service/hw_cluster/` — whole directory (3352 LOC including 1172 LOC of tests that move to the CLI)
- `crates/manta-server/src/server/handlers/hw_cluster.rs` — 205 LOC
- `crates/manta-shared/src/types/api/hw_cluster.rs` — whole file
- `pub mod hw_cluster;` line in `crates/manta-shared/src/types/api/mod.rs`
- 3 route registrations in `crates/manta-server/src/server/mod.rs` (router mounts)
- 4 schema registrations in `crates/manta-server/src/server/api_doc.rs`
  (`AddHwComponentRequest`, `DeleteHwComponentRequest`, `ApplyHwConfigurationRequest`,
  `HwClusterMode`)
- `comfy-table = "7.2.2"` line in `crates/manta-server/Cargo.toml`

The CLI's `crate::openapi_client::types::{HwClusterMode, AddHwComponentRequest,
DeleteHwComponentRequest, ApplyHwConfigurationRequest}` and the
`client.openapi.{add_hw_component, delete_hw_component, apply_hw_configuration}`
methods disappear in the next progenitor regen — automatic, no manual edits.

## Authorization

No changes. Verified during design:

- `GET /api/v1/groups/hardware` — `service::hardware::get_hardware_cluster`
  calls `validate_user_group_vec_access` at the service layer, so per-group
  authz is enforced even though the handler does not explicitly call it.
- `GET /api/v1/groups/{name}` — used by the new delete flow to check
  target existence; same service-layer authz applies.
- `POST /api/v1/groups` — admin-only (`is_user_admin`). Today's `--create-target-group`
  flow is also effectively admin-only because `validate_user_group_access`
  rejects non-existent groups for non-admins. No regression.
- `POST /api/v1/groups/{name}/members` (add-only, unchanged) — handler
  calls `validate_user_group_access` explicitly.
- `DELETE /api/v1/groups/{name}/members` (remove-only, unchanged) — same.
- `DELETE /api/v1/groups/{name}` (delete group) — handler calls
  `validate_user_group_access`. Used by `--delete-empty-parent-group`.
- **New** `PUT /api/v1/groups/{name}/members` — same shape as the existing
  `POST`/`DELETE` handlers on this path: `validate_user_group_access(name)`
  on the group, then `validate_user_group_members_access(touched_xnames)`
  on the union of added + removed nodes.

No audit-trail changes: `crates/manta-server/src/server/common/audit.rs`
only emits auth events (`send_auth_audit`). No per-mutation audit exists
to lose.

## Score-table output channel

Today: `tracing::info!("\n{table}\n")` — routes to server stderr.

After migration: `eprintln!("\n{table}\n")` from the CLI — also stderr.

Stderr is the right channel because:

- It matches operator expectation (today's tracing default is stderr).
- It keeps stdout clean for `--output json` consumers that pipe through `jq`.
- No mode-gating needed (no `if output == "json" { skip }`).

ANSI color codes (`Color::Green`/`Yellow`/`Red`) emit unconditionally,
matching the rest of `crates/manta-cli/src/output/` which performs no
TTY detection (verified by grep — no `is_terminal`/`atty`/`force_no_color`
calls). A global `--no-color` flag is a follow-up if it ever becomes a real
ask.

## Test strategy

### Existing tests that port verbatim

All 34 tests in `crates/manta-server/src/service/hw_cluster/tests.rs`
move to `crates/manta-cli/src/hw_cluster/tests.rs`. Verified by grep:
zero references to `infra`, `backend`, `InfraContext`, or any mock —
they construct `HashMap`s and `Vec`s of synthetic data and assert pure
function output. Only edit needed: `use super::scoring::{…}` paths.

### One new fixture test

The single piece of new code in the migrated module is the
`NodeSummary → (Vec<processor_models>, Vec<accelerator_models>,
Vec<memory_mib>)` shim that replaces the 214-line `hw_inventory_utils.rs`
JSON-pointer parser. A fixture test in `tests.rs` covers it:

- Build a representative `NodeSummary` (a couple of processors, one accelerator,
  two DIMMs of known capacity, including at least one `ArtifactSummary { info: None }`
  entry to pin down the null-filtering behaviour).
- Run it through the new shim.
- Assert against a hand-computed expected `HashMap<String, usize>`.

The hand-computed expectation is the durable reference. Cross-checking
against the old JSON-pointer parser is useful while step 2 of the cutover
runs (the old code still exists) but not load-bearing: the spec is
the expected behaviour, not whatever the old code happened to do.

### Server-side coverage for the new endpoint

`crates/manta-server/src/server/handlers/group.rs` gets a handler test
for `PUT /groups/{name}/members`:

- Happy path: PUT with a member set, verify response `added`/`removed`/`final_members`
  match the diff against the prior state.
- Non-admin, group not in accessible set → 400 (mirrors POST/DELETE behaviour).
- Non-admin, group accessible but a touched xname not in accessible members → 400.
- Empty `members` array → group ends up empty; success.

### CLI end-to-end test

One integration test in `crates/manta-cli/tests/` that exercises a Pin
and an Unpin against a stub OpenAPI server (an `axum::Router` wired up
in-test, similar in style to the existing CLI integration tests). Asserts:

- Exactly the expected sequence of REST calls (URLs, methods, request bodies).
- Final `target_nodes` and `parent_nodes` match a hand-computed expected
  partition for a small synthetic fixture.
- Re-running the same Pin against the resulting state is a no-op
  (`added: []`, `removed: []` in the PUT response).

### Tests that get deleted

Cases in `crates/manta-server/tests/integration.rs` and
`crates/manta-server/tests/server_routes.rs` exercising the 3 removed
endpoints. Count to be determined during execution; flagged as a TODO
step in the cutover order, not a blocking unknown.

## Cutover order

1. **Add `PUT /groups/{name}/members`**: wire types, service function,
   handler, router mount, OpenAPI schema registration, server-side tests.
   Server builds and tests green at this point. Endpoint exists but no
   client uses it yet.
2. **Create `crates/manta-cli/src/hw_cluster/`** with `mod.rs`, `scoring.rs`,
   `pin_unpin.rs`, `apply.rs`, and `tests.rs`. Local `HwClusterMode` enum.
   `pub mod hw_cluster;` added to `crates/manta-cli/src/lib.rs` (or
   `main.rs` — whichever currently declares the module set).
   Unit tests + new fixture test pass in isolation. CLI still compiles
   against the unchanged `openapi_client` (the old hw_cluster methods are
   still present, just unused).
3. **Rewrite the 3 dispatchers** (`dispatch/apply/hardware_group.rs`,
   `dispatch/add/hardware.rs`, `dispatch/delete/hardware.rs`) to call
   `crate::hw_cluster::*` instead of `client.openapi.{apply,add,delete}_hw_*`.
   `HwClusterMode` import swaps from `crate::openapi_client::types`
   to `crate::hw_cluster`. CLI builds clean.
4. **Delete server-side hw_cluster code**:
   `service/hw_cluster/`, `handlers/hw_cluster.rs`, `manta-shared/src/types/api/hw_cluster.rs`.
   Remove 3 routes + 4 schema entries from `server/mod.rs` and `api_doc.rs`.
   Remove `pub mod hw_cluster;` from `manta-shared/src/types/api/mod.rs`.
   Server builds clean.
5. **Regenerate `crates/manta-cli/openapi.json`**:
   `cargo run -p manta-server -- --emit-openapi > crates/manta-cli/openapi.json`.
   Commit the regenerated file. (Adds `replace_group_members`; drops the 3
   hw_cluster methods + `HwClusterMode` + 3 request bodies from the spec.)
6. **Rebuild CLI**. progenitor regenerates `openapi_client`:
   `client.openapi.replace_group_members(...)` is now available; dispatchers
   updated in step 3 use it directly (the new module's `apply_group_updates`
   is the only call site).
7. **Drop `comfy-table` from `crates/manta-server/Cargo.toml`**. `cargo update`.
   `cargo tree -p manta-server | grep -E 'comfy|crossterm'` should return nothing.
8. **Test sweep**: drop the 3 endpoint cases from
   `crates/manta-server/tests/integration.rs` and `tests/server_routes.rs`
   (identify exact case names first). Add `replace_group_members` integration
   coverage. Add the CLI end-to-end test described above.
9. **CHANGELOG**: "added `PUT /api/v1/groups/{name}/members` for atomic
   member replacement; moved hardware-cluster pin/unpin algorithm from
   manta-server to manta-cli; removed the
   `/api/v1/hardware-clusters/{target}/{members,configuration}` endpoints."

## Risks

- **`NodeSummary` shim correctness.** One new code path (~30 LOC) replaces
  214 lines of JSON-pointer parsing. The differences are subtle: e.g.,
  a processor with `Model: null` is dropped by the old `filter_map`
  but produces an `ArtifactSummary { info: None, .. }` from `NodeSummary::from_csm_value`.
  The fixture test in `tests.rs` is the only practical guard against
  this changing scoring output. *Mitigation:* fixture test is mandatory
  in step 2.
- **Concurrent group edits between read and write — slight regression vs
  today.** Both flows have a race window between reading current
  membership and applying the update. They are not equivalent:

  - **Today**: `apply_hw_configuration` passes explicit `remove[]` and
    `add[]` to `update_group_members`. A node added by a third party
    between our read and our call is *preserved* — we never asked to
    remove it.
  - **New PUT-replace**: the CLI sends a final-state `members: [...]`.
    Server reads current state, diffs against `members`, and removes
    anything not in our list — including the third party's just-added
    node. The third party's removal of a node we expected to be present
    is *also* clobbered (server's diff says "add it back").

  Trade-off: simpler API at the cost of write-side authority. The
  pin/unpin use case is a planned admin operation, not a hot loop, so
  the practical exposure is low. The PUT response's `added`/`removed`
  fields surface the clobber to the CLI, which renders them in the final
  output — operators see what was changed, including anything unexpected.
  *Mitigation:* none specific; documented honestly. If clobber becomes a
  real-world pain point a follow-up can switch the endpoint shape from
  replace to diff (`add[]`/`remove[]`) without changing call counts.

- **`PUT /groups/<name>/members` request-body size**. The design's
  `members: Vec<String>` carries the full final member list. A pattern
  touching hundreds of xnames produces a JSON body in the low tens of
  kilobytes — well under axum's default 2 MiB body limit. *Mitigation:*
  no action needed; flagged for awareness.
- **Existing server integration tests are not yet inventoried.** The exact
  count of test cases referencing the 3 removed endpoints in
  `crates/manta-server/tests/integration.rs` and `tests/server_routes.rs`
  is unknown without reading. *Mitigation:* step 8 reads first, then
  prunes. Not blocking; just unmeasured.

## Out of scope

- Relaxing `POST /api/v1/groups`'s admin-only check to allow group-namespace-based
  creation. This was considered earlier as a way to "preserve" today's
  `--create-target-group` for non-admins, but analysis showed today's
  flow is also effectively admin-only (via `validate_user_group_access`
  rejecting non-existent groups). Not a regression to mitigate; not in
  this PR. If it lands later as a UX improvement, it's a one-line change
  to one handler.
- A `--no-color` global flag for the CLI. Stderr ANSI sequences from
  `print_score_table` match the rest of the CLI's color behaviour today.
  Not migration scope.
- Replacing `manta_backend_dispatcher::types::NodeSummary` and the
  CSM-specific JSON pointer paths with a backend-agnostic intermediate
  type. The migration consumes whatever `NodeSummary` shape the CLI's
  generated openapi client produces; tightening that contract is a
  separate concern tracked in the workspace split epic.
