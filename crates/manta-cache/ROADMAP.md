# manta-cache — roadmap

> **Status:** Stages 1 + 2 delivered (collapsed), Stage 3 delivered, Stage 4 partially delivered. The core library exists as the standalone `manta-cache` crate (Stage 1 was implemented directly as the crate — no compile-time dependency on `manta-server`, so the intermediate module step bought nothing); the Stage-3 HTTP wrapper exists as the `crates/manta-cache-server` binary (standalone shared service, per the decision recorded in Stage 3 below); and the Stage-4 management **refresh** endpoints are live. What remains of Stage 4: the CLI-side pre-resolution integration (decided, not yet built — see Stage 4), the conflict policy, and the descoped site-CRUD question.

For background — what manta is, what a "site" means, what HSM groups are, and why a cache helps — see the sibling [README.md](README.md).

Four stages. Each stage produces a self-contained deliverable that can be merged, reviewed, and used on its own; nothing later is a hard prerequisite for the user-visible payoff of an earlier stage.

```text
  Stage 1            Stage 2            Stage 3            Stage 4
  ────────           ────────           ────────           ─────────────────
  rust module    ──> extract into   ──> HTTP API       ──> management ops,
  inside             manta-cache        wrapped              integration with
  manta-server       crate              around crate         manta-server,
  / manta-shared                                             persistence,
                                                             conflict policy
```

## Stage 1 — Rust module inside `manta-server` or `manta-shared`

Implement the cache as a **private module of an existing crate**. No new crate yet; the source lives at e.g. `crates/manta-server/src/cache/` or `crates/manta-shared/src/cache/`. Choosing between the two is a judgement call: `manta-server` is the natural home because the cache is server-side logic, but `manta-shared` is preferable if there is any prospect of the CLI consuming it directly. The recommendation is `manta-server` until proven otherwise.

What the module ships:

- Data types — `Site`, `Group`, `Members`, and the combined index struct that owns the two derived maps (`group → site`, `xname → site`).
- An async `refresh(sites: &[SiteDescriptor]) -> Result<Index, CacheError>` that fans out one HTTP call per site to populate the index. `SiteDescriptor` carries `{ name, manta_server_url, token }`.
- Synchronous lookup methods on `Index`: `group_to_site(label) -> Option<&str>`, `xname_to_site(xname) -> Option<&str>`, `sites() -> impl Iterator`.
- Unit tests that exercise the lookup methods against fixture inputs (no live `manta-server` needed). The starter fixture is [`testdata/groups-prealps.json`](testdata/groups-prealps.json) — a real extract of `GET /v2/groups` from the CSCS **prealps** test site; see the [Mock fixture](#mock-fixture-for-offline-tests) section below for details.
- A single integration test that runs `refresh` against one live `manta-server` and asserts the index shape — gated behind an env var so CI without a backend skips it.

No public API stability promise — the module is internal-only. Other code in the same crate may import it; nothing outside the crate sees it.

**Acceptance.** Module compiles, unit tests pass, the integration test passes against a known-good `manta-server` URL + token.

> **✅ Delivered** (built directly as the crate — see Stage 2). The library exposes `SiteDescriptor`, `Index` (`group_to_site` / `xname_to_site` / `groups` / `sites` / `group_members`), and `async refresh()`. `refresh` is **2 calls per site** (`GET /groups/available` + `GET /groups/nodes`, the latter unfiltered so one call returns every node and its `hsm` membership), fanned out concurrently and tolerant of per-site failure: it returns a `RefreshOutcome` holding the index over the sites that answered plus one error per site that did not. Errors are a crate-local `CacheError` (thiserror). Unit tests cover the lookups + builder (incl. multi-group membership and the cross-site collision rules); the integration test is env-gated (`MANTA_CACHE_IT_*`) and skips when unset. Offline wiremock tests (`tests/mock_server.rs`) cover the refresh HTTP path — per-site headers, the two-call fan-out, and failure mapping — with no live server. The fixture drives an offline test (`tests/fixture.rs`) through the public snapshot constructor `Index::from_snapshots`; note the fixture is `GET /groups`-shaped (not `/groups/available`, which returns a bare list of names) — the *refresh path* still uses the two-call source pending the refresh-source [open question](#open-questions).

## Stage 2 — Extract into the `manta-cache` crate

Promote the Stage-1 module into an independent workspace member at `crates/manta-cache/`. **This is the first time the crate physically exists**; until this stage lands, the directory only carries this roadmap and the sibling README.

Steps:

- `git mv` the Stage-1 module's source files from `manta-server/src/cache/` (or wherever Stage 1 placed them) into `crates/manta-cache/src/`.
- Add `crates/manta-cache/Cargo.toml` with the workspace-inherited metadata pattern the other crates use (`version.workspace = true`, etc.).
- Register the crate in the workspace `Cargo.toml`'s `members` array.
- Define the **public** API surface: the same lookup + refresh methods Stage 1 already shipped, but now exposed via `pub` and documented in the crate root.
- Update `manta-server` to depend on `manta-cache = { workspace = true }` and switch its call sites from the in-place module to the crate's public API.

**Acceptance.** `cargo build -p manta-cache` succeeds standalone, `manta-server` builds against it, and the existing `manta-server` behaviour is unchanged (the cache is still in-process; only its source-tree location moved).

> **✅ Folded into Stage 1.** The crate was created up front, so there was no module to `git mv`. The crate is registered in the workspace `members` array and `cargo build -p manta-cache` succeeds standalone (it depends only on `reqwest`/`serde`/`thiserror`/`futures`/`tracing`, none of the sibling repos). It is `publish = false` and **not yet consumed** by `manta-server`; the `manta-cache = { workspace = true }` dependency + call-site wiring land with the Stage-4 integration.

## Stage 3 — HTTP APIs

Wrap the crate in a small HTTP service so the cache can be queried **from outside its host process**. This is what makes "one cache shared by several `manta-server` instances" possible — until now every `manta-server` would have to build and hold its own copy.

Endpoints, draft (final shapes TBD during implementation):

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/sites` | List cached sites |
| `GET` | `/lookup/group/{label}` | Resolve `group → site` |
| `GET` | `/lookup/nodes?xnames=…` | Resolve a comma-separated xname list → site(s) |

**Deployment shape — decided (2026-07, with the maintainer): standalone shared service.** A separate `manta-cache-server` binary that every `manta-server` instance points at. The deciding factor is the long-term plan to use the cache from other projects (OpenCHAMI tooling among them): a consumer outside this repo rules out in-process endpoints on `manta-server`. The multi-project features themselves are out of scope for now (and may eventually argue for extracting the crate to its own repo); the shape decision is what matters here. For the record, the rejected alternatives were:

1. ~~**In-process endpoints on `manta-server`**~~ — the cache as more routes on the existing Axum router. Ruled out by the multi-project consumer.
2. ~~**Sidecar binary**~~ — one `manta-cache-server` colocated 1:1 with each `manta-server`. Nothing stops an operator from deploying the standalone binary this way, but the *design target* (config, auth, TLS) is the shared service.

**Authentication — decided (2026-07): service-account-style token per site.** The cache holds one rotated, scoped token per site; all users share the resulting index; per-user authorisation continues to run in the `manta-server` handler that ultimately receives the resolved request. The per-user-cache alternative (index built with the caller's own token, partitioned per `(user, site)`) was rejected: for a deployment shared by many users it multiplies refresh traffic and memory by the user count for little gain, since the cache only routes.

Two **sub-decisions surfaced by the shape choice** remain for the Stage-3 implementer:

- **Service-account token sourcing** — where the per-site tokens come from (config file, environment, Vault) and how rotation reaches the running service.
- **Securing the cache's own endpoints** — as a shared network service, the lookup API needs its own TLS + caller-auth story (the in-proc and Unix-socket shapes would have sidestepped this).

**Acceptance.** A `curl` from outside the host process can query the three endpoints; the shape and content of the responses match what the in-process API returned at Stage 2.

> **✅ Delivered** as the `crates/manta-cache-server` binary. What shipped, and the sub-decisions taken:
>
> - **Endpoints** (under an `/api/v1` prefix, matching manta conventions rather than the bare draft paths above): `GET /api/v1/sites` → `["alps", …]`; `GET /api/v1/lookup/group/{label}` → `{"site": "…"}` or 404; `GET /api/v1/lookup/nodes?xnames=…` → `{"site": <unanimous-or-null>, "resolutions": {xname: site}, "unknown": [xname]}` (400 only for a missing/empty `xnames` param — split or partially-unknown lists are stated in the 200 payload for the Stage-4 caller to police). Plus an unauthenticated `GET /health` for probes.
> - **Config**: `cache-server.toml` in the shared manta config dir (`MANTA_CACHE_SERVER_CONFIG` to override), `[server]` + `[sites.<name>]` blocks; the bootstrap mirrors `manta-server` (hand-rolled flags, fail-closed TLS unless `allow_http`, graceful SIGTERM drain, startup summary). Default ports 8444 (TLS) / 8081 (plain) — one above manta-server's, so a colocated pair never collides.
> - **Token sourcing** (sub-decision): per site, exactly one of inline `token` or `token_file`; `token_file` is re-read on every refresh, so a secret manager can rotate credentials without a restart.
> - **Securing the cache's endpoints** (sub-decision): TLS fail-closed like manta-server, plus an optional `[server] api_token` shared bearer secret guarding `/api/v1/*` (absent = rely on network controls). Revisit for something stronger if the multi-project consumers need it.
> - **Refresh lifecycle**: initial refresh before the listener binds; per-site failures are tolerated (failed sites are absent, warned about, and retried by the optional `refresh_interval_secs` periodic loop, which swaps the index wholesale — keeping a failed site's stale entries alive is Stage-4 persistence territory). On-demand refresh endpoints stay in Stage 4.

## Stage 4 — Management functionalities + manta-server integration

The final stage delivers the user-visible payoff and the operability surface needed to run the cache in production.

**Management endpoints.** Round out the API so the cache can be operated without restarts:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/refresh` | Full re-sync of every site |
| `POST` | `/api/v1/refresh/{site}` | Re-sync one site |

> **✅ Refresh endpoints delivered** in `manta-cache-server`. Both **require** a configured `[server] api_token` and answer `403` when none is set — a refresh triggers a cross-site HTTP fan-out, so unlike the read-only lookups it must never be an open amplification lever. The server keeps the last-good `SiteSnapshot` per site: `POST /refresh/{site}` re-fetches one site and rebuilds the index from that plus the stored siblings, and on failure the previous state (including that site's last-good snapshot) keeps serving. The full `POST /refresh` replaces the snapshot store wholesale, matching the startup/periodic semantics.

The originally-drafted **site CRUD** (`POST /sites`, `PUT /sites/{name}`, `DELETE /sites/{name}`, with mutations persisted back to the config file) is **descoped to an open question** rather than implemented: in the deployment the standalone-service decision implies, configuration is declarative and often a read-only mount, and a service writing service-account credentials back into its own TOML is the part of this draft most likely to be wrong. Revisit with the maintainer if runtime site management is needed at all.

**Integration — decided (2026-07, with the maintainer): CLI-side pre-resolution.** The pre-Stage-3 draft had `manta-server` consult the cache when `X-Manta-Site` was absent, but that founders on per-site tokens: a CSM token is issued by one site's Keycloak, and the CLI chooses which cached token to attach *based on the site* — the very thing being resolved. Even a perfect server-side resolution would see the request arrive bearing the wrong site's credential. Instead, the **`manta` CLI consults the cache service before dispatch**:

1. Explicit `--site` / `cli.toml` `site` — honored as today; the cache is not consulted.
2. The command names a group → `GET /api/v1/lookup/group/{label}` on the cache → site.
3. The command names xnames → `GET /api/v1/lookup/nodes?xnames=…` → the unanimous site, or a clear client-side error listing the per-xname resolutions when the list splits across sites or contains unknowns.
4. Neither → error as today (site is required).

From the resolved site onward everything is unchanged: the right per-site token cache, the right `X-Manta-Site` header, per-user authorisation in the `manta-server` handler. The CLI grows two `cli.toml` keys (`cache_url`, optional `cache_api_token`) and degrades gracefully — cache unreachable means "site required", exactly today's behaviour. `manta-server` needs **no change** for this design.

**Security stance (decided 2026-07).** The lookup endpoints serve read-only routing metadata — site names, group labels, xnames; no member lists, credentials, or per-node state — considered non-sensitive inside the deployment perimeter. They are open by default, gated by the optional shared `api_token`, and protected primarily by network placement. Two deliberate caveats: the management endpoints always require the token (above), and TLS on the cache matters for **integrity** more than secrecy — a tampered resolution answer could steer a destructive command at the wrong cluster, so production deployments should still terminate the cache behind TLS.

**Lifecycle.** (The pre-Stage-3 draft had `manta-server` instantiating the cache in-process; superseded.) The standalone `manta-cache-server` owns its own lifecycle — initial refresh before its listener binds, optional periodic refresh, on-demand management refresh. `manta-server` is not involved; the CLI treats the cache as an optional accelerator and falls back to explicit `--site` when it is absent.

**Cross-cutting concerns** decided in this stage:

- **Conflict policy** when a group label or xname appears at more than one site (reject, prefer a default site, or return all candidates and let the caller disambiguate). Still open — and the built `Index` currently does not retain the fact that a collision happened, so surfacing candidates needs library support too.
- **TTL / freshness**. The force half exists (`POST /refresh/{site}` re-syncs a known-mutated site without waiting for the timer); an optional per-site stale window remains open.
- **Persistence**. Decide whether to persist the index to disk (sqlite / JSON snapshot) so restarts skip the cross-site fan-out. May be in scope for Stage 4 or punted to a follow-up.

**Acceptance.** An operator can run a `manta` command that names only a group or only a node list, with no `--site`, and it reaches the right cluster (via the CLI-side pre-resolution above). Failure modes (cross-site xname list, unknown group) produce clear errors naming the per-xname resolutions. The refresh half of runtime management is already delivered; site CRUD is not part of the acceptance unless the open question above resolves in its favour.

---

## Testing the manta-server integration

This section describes the end-to-end test path that exercises the cache against a real `manta-server`, and the prerequisites a tester needs to run it locally. It complements the unit and integration tests of each stage; the focus here is the Stage-4 wiring — i.e. that an incoming request without an explicit `X-Manta-Site` is resolved through the cache to the right site.

### Integration shape under test

Per the CLI-side pre-resolution decision (Stage 4 above), the cache is consulted by the **`manta` CLI** before it dispatches to `manta-server`. The flow is:

1. The operator runs a command with no `--site` and no `site` in `cli.toml` (e.g. `manta power off group compute` or a command naming xnames).
2. The CLI asks the cache: group label → `GET /api/v1/lookup/group/{label}`; xname list → `GET /api/v1/lookup/nodes?xnames=…`.
3. With the resolved site in hand, the CLI proceeds exactly as if `--site <resolved>` had been given: right per-site token cache, right `X-Manta-Site` header. A split or unknown resolution is a client-side error before any `manta-server` call.
4. `manta-server` behaves as today — it still receives an explicit site header on every request.

### Local test setup

The integration can be exercised against a locally running `manta-cache-server` + `manta-server` pointed at the real CSCS test sites. The cache runs its initial refresh before its listener binds, so both prerequisites below must be satisfied **before** `manta-cache-server` is launched — otherwise the affected sites fail their refresh and are simply absent from the index (reported in the startup warnings, recoverable via `POST /api/v1/refresh`).

- **VPN access to the test sites.** The startup refresh and `manta-server`'s backend calls both reach CSM / OpenCHAMI endpoints that live on the internal network; without VPN, the refresh fails at boot and no lookups succeed.
- **Keycloak roles on the test HSM groups.** The tester's Keycloak account must carry the roles that grant read/operate access to the HSM groups used in the test scenarios (e.g. the `nodes_free` and equivalent test-only groups on each site). Without the right roles, `manta-server` returns `403` even after the cache has resolved the site correctly, which masks whether the cache itself is working.

### Test scenarios

Once the above is in place, the minimum scenarios to walk through are:

1. **Group-only request.** Issue a `manta` command that names only a group label present at exactly one site, with `site` removed from `cli.toml`. Expect: command reaches the correct cluster; the CLI's log/verbose output shows the site came from the cache lookup, not from config.
2. **xname-only request, single site.** Issue a command that names xnames all belonging to the same site. Expect: same as above.
3. **xname-only request, split sites.** Issue a command whose xname list straddles two sites. Expect: a clear client-side error listing the per-xname `(xname, site)` resolutions; no `manta-server` call made.
4. **Explicit site wins.** Issue a command with `--site` set to a site that *does not* own the named group. Expect: the explicit site is honored, the cache is not consulted, and the backend returns whatever error it would for an unknown group at that site. This confirms the cache has not silently overridden the explicit choice.
5. **Unknown group / unknown xname.** Issue a command naming a label or xname the cache has never seen. Expect: a clear "no site found for …" error from the CLI (and a hint to pass `--site` explicitly).
6. **Cache down.** Stop `manta-cache-server` and repeat scenario 1. Expect: the CLI degrades to today's "site is required" error — the cache is an accelerator, not a dependency.

### Mock fixture for offline tests

For unit tests and any scenario that should not require VPN / Keycloak / a live `manta-server`, the crate ships a captured response under [`testdata/groups-prealps.json`](testdata/groups-prealps.json). It is the verbatim payload returned by `GET /v2/groups` against the CSCS **prealps** test site (`/v2/groups/available` returns only a bare `Vec<String>` of names — this fixture carries full group objects with `label` + `members.ids`). It drives the offline test `tests/fixture.rs`, which folds it into a `SiteSnapshot` and builds the index through the public `Index::from_snapshots` constructor.

Notable properties of this fixture, useful when writing assertions:

- **Site-umbrella group.** `prealps` itself is a group whose membership is the full xname list at that site. Treat it as the cluster-wide group.
- **Tenant / exclusive groups.** `k3s_server` and `k3s_agent` share `exclusiveGroup: "k3s"`; `cscs` carries `exclusiveGroup: "tapms-exclusive-group-label"` and a `tags` entry. The cache must tolerate (or ignore, depending on scope) these optional fields.
- **Overlapping membership.** A single xname appears in multiple groups (e.g. `x8000c1s5b1n0` is in `prealps`, `rotondo`, `cavel`, and `cavel_gh`). The `xname → site` derived index will therefore collapse many group entries onto the same site — this is the expected steady state, not a duplicate-key bug.
- **Empty groups.** `cavel_arm` and `k3s_agent` have empty `members.ids`. Lookups against their labels must still resolve the *site*, even though no xname lookup will ever land on them.
- **Single-site scope.** This fixture represents one site. To exercise the cross-site conflict path (Stage 4, scenario 3 in [Test scenarios](#test-scenarios)) a second fixture from a different site is needed; capture one when that scenario is implemented and place it alongside as `testdata/groups-<site>.json`.

### Out of scope for this test pass

- Validating the management endpoints (`POST /refresh`, site mutation) — covered by the Stage-4 management-API tests, not the integration walkthrough.
- Performance / cadence of refresh — see the refresh-cadence [open question](#open-questions).
- Per-user authorisation — out of scope by design (the cache is a routing layer; authorisation runs in the downstream handler).

---

## Open questions

These are the decisions the roadmap deliberately punts on until the stage that actually needs them. Listed once here so they are easy to find and revisit.

| Question | Decide at | Why deferred |
|---|---|---|
| ~~`manta-server` vs `manta-shared` as Stage-1 home~~ | ~~Stage 1 kickoff~~ | **Resolved:** standalone crate — no compile-time dep on either (HTTP-only). |
| ~~Deployment shape: in-proc / sidecar / standalone~~ | ~~Stage 3~~ | **Resolved (2026-07): standalone shared service** — planned reuse from other projects (OpenCHAMI) rules out in-proc. See Stage 3 for the rationale and the two sub-decisions it surfaces (token sourcing, securing the cache's own endpoints). |
| ~~Auth model: service-account vs per-user~~ | ~~Stage 3~~ | **Resolved (2026-07): service-account-style token per site**; shared index, per-user authorisation stays in the downstream `manta-server` handler. |
| Conflict policy when label / xname spans sites | Stage 4 | Only matters once an integration layer needs to *resolve* something to a single site. Stage 1 ships documented deterministic rules (see `Index`'s "Conflict handling" doc), not a policy — and the built `Index` does not retain that a collision happened, so surfacing candidates needs library support. |
| ~~Integration architecture: server-side vs CLI-side resolution~~ | ~~Stage 4~~ | **Resolved (2026-07): CLI-side pre-resolution** — per-site Keycloak tokens make server-side resolution self-defeating (the request would carry the wrong site's credential); see Stage 4. |
| Site CRUD endpoints (runtime add/update/delete + config persistence) | Stage 4, if at all | Descoped from the original draft: declarative/read-only config deployments sit badly with a service rewriting its own TOML (and tokens). Needs a maintainer decision on whether runtime site management is wanted. |
| Persistence (in-memory vs on-disk snapshot) | Stage 4 | A cold start is a few HTTP calls per site; tolerable until the deployment shape pushes back. |
| Refresh cadence (pull-on-demand vs periodic background) | Stage 4 | Depends on the deployment shape and traffic pattern. |
| Refresh source: two calls (`/groups/available` + `/groups/nodes`) vs one `GET /groups` | Next refresh change | Verified server-side: unfiltered `GET /groups` is access-scoped identically to `/groups/available` (both go through `resolve_target_and_available_groups`) and returns labels + members together — one atomic call per site, whose shape matches the checked-in fixture (which could then drive a unit test). Trade-off: loses labels known only from nodes' `hsm` fields. |
| ~~Partial-failure tolerance of `refresh`~~ | ~~Stage 3, at the latest Stage 4~~ | **Resolved:** `refresh` returns a `RefreshOutcome { index, failures }` — the index covers every site that answered; each failed site contributes a `CacheError`. One unreachable site no longer costs the whole refresh (or, under Stage 4's "refresh before the listener starts" lifecycle, `manta-server` startup). What *policy* a caller applies to an incomplete outcome (serve partial vs refuse to start) is decided with the Stage-4 integration. |
| ~~In-process population path (public snapshot constructor)~~ | ~~Stage 3~~ | **Resolved:** `SiteSnapshot`, `NodeMembership`, and `Index::from_snapshots` are public, so an embedding `manta-server` can populate the cache from its own service layer without HTTP (defusing the in-proc chicken-and-egg: the in-proc shape cannot HTTP-refresh against its own not-yet-listening router at startup). `tests/fixture.rs` exercises this path with the prealps capture. |

## Decisions taken at Stage 1

1. **Home — neither `manta-server` nor `manta-shared`: a standalone crate.** Because the cache talks to `manta-server` purely over HTTP, it has no compile-time dependency on either crate, so the cleanest home is its own workspace member from the start (collapsing Stage 2).
2. **Wire types — self-contained, no `manta-shared` dependency.** The cache reads only `xname` + `hsm` from `GET /groups/nodes`, so it defines a minimal local deserialisation struct instead of pulling `manta-shared` (and transitively `manta-backend-dispatcher` + config/utoipa/…) in for a 12-field type. serde ignores the unused fields; the drift risk on these core identifiers is covered by the integration test. Mirrors the existing `dto.rs` "small mirror over heavy dep" choice. Reversible if a future stage wants the shared type.
3. **Stage 3 (deployment shape, auth model) — deferred** to that stage as the open-questions table records; nothing about it needed locking to ship Stage 1.
