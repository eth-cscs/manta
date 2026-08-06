# manta-cache — roadmap

> **Status:** Stages 1 + 2 delivered (collapsed), Stage 3 delivered, Stage 4 mostly delivered, Stage 5 delivered. The core library exists as the standalone `manta-cache` crate (Stage 1 was implemented directly as the crate — no compile-time dependency on `manta-server`, so the intermediate module step bought nothing); the Stage-3 HTTP wrapper exists as the `crates/manta-cache-server` binary (standalone shared service, per the decision recorded in Stage 3 below); the Stage-4 management **refresh** and **dump** endpoints are live; the **CLI-side pre-resolution integration is built** (`cache_url` in `cli.toml` — see Stage 4); and Stage 5 moved the refresh onto **per-site service accounts**. What remains: the conflict policy, the descoped site-CRUD question, and the optional stale-window/persistence items. The full chain was verified live against prealps on 2026-07-19 — see [LIVE-TEST.md](LIVE-TEST.md) for the reproducible walkthrough.

For background — what manta is, what a "site" means, what HSM groups are, and why a cache helps — see the sibling [README.md](README.md).

Five stages. Each stage produces a self-contained deliverable that can be merged, reviewed, and used on its own; nothing later is a hard prerequisite for the user-visible payoff of an earlier stage.

```text
  Stage 1            Stage 2            Stage 3            Stage 4              Stage 5
  ────────           ────────           ────────           ─────────────────    ────────────────
  rust module    ──> extract into   ──> HTTP API       ──> management ops,  ──> production
  inside             manta-cache        wrapped              integration with     credentials
  manta-server       crate              around crate         manta-server,        (service
  / manta-shared                                             persistence,          accounts)
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
> - **Token sourcing** (sub-decision): per site, exactly one of inline `token` or `token_file`; `token_file` is re-read on every refresh, so a secret manager can rotate credentials without a restart. **Superseded by Stage 5**, which removed inline `token` — `token_file` is now the only form — and added the credential advisory. *Where the tokens come from* (a Keycloak service account per site) was left open here and is settled there.
> - **Securing the cache's endpoints** (sub-decision): TLS fail-closed like manta-server, plus an optional `[server] api_token` shared bearer secret guarding `/api/v1/*` (absent = rely on network controls). Revisit for something stronger if the multi-project consumers need it.
> - **Refresh lifecycle**: initial refresh before the listener binds; per-site failures are tolerated (failed sites are absent, warned about, and retried by the optional `refresh_interval_secs` periodic loop, which swaps the index wholesale — keeping a failed site's stale entries alive is Stage-4 persistence territory). On-demand refresh endpoints stay in Stage 4.

## Stage 4 — Management functionalities + manta-server integration

The final stage delivers the user-visible payoff and the operability surface needed to run the cache in production.

**Management endpoints.** Round out the API so the cache can be operated without restarts:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/refresh` | Full re-sync of every site |
| `POST` | `/api/v1/refresh/{site}` | Re-sync one site |
| `GET` | `/api/v1/dump` | Debugging dump of everything cached |

> **✅ Refresh endpoints delivered** in `manta-cache-server`. Both **require** a configured `[server] api_token` and answer `403` when none is set — a refresh triggers a cross-site HTTP fan-out, so unlike the read-only lookups it must never be an open amplification lever. The server keeps the last-good `SiteSnapshot` per site: `POST /refresh/{site}` re-fetches one site and rebuilds the index from that plus the stored siblings, and on failure the previous state (including that site's last-good snapshot) keeps serving. The full `POST /refresh` replaces the snapshot store wholesale, matching the startup/periodic semantics.

> **✅ Dump endpoint delivered** (`GET /api/v1/dump`) — a debugging view of the whole cache, for humans and `jq`. Not consumed by `manta-server` or the CLI. JSON only. What it answers, and the decisions behind each part:
>
> - **`groups` + `xnames` are a bulk mirror of the lookup endpoints** — same owners, same member lists, no second resolution path. The guiding rule was that the dump must be a trustworthy oracle for "what would the CLI have resolved", so it deliberately serves the derived `Index` rather than the raw snapshots.
> - **`sites` is keyed by *configured* site, not indexed site.** A site whose refresh failed is absent from the index, and its absence is exactly what a reader is trying to explain — so each entry carries `manta_server_url`, `token_file` (the path, **never** the token; Stage 5 replaced the earlier `token_source` enum when inline tokens went away), `credential` (Stage 5's advisory block), `in_index`, `refreshed_at` + `age_seconds`, and `last_error`. This required new per-site state: `CacheState.status`, written by both refresh paths. The two fields are independent — a failed `POST /refresh/{site}` records the error while keeping the previous snapshot's timestamp, because that data is still what's serving.
> - **`conflicts` is the one thing that is not a mirror.** `Index` resolves collisions at build time and keeps only the winner, so a group whose members were discarded when another site claimed the label is indistinguishable from a genuinely small group. Since the server holds the snapshots anyway, the dump re-scans them per request and reports contested labels/xnames with `claimed_by` (every site that contributed), `listed_by` (the subset that listed the label in `/groups/available`, which is what the tie-break actually runs over), `owner`, the tie-break `reason`, and `discarded_members` — **the xnames missing from the served member list**, so it is `[]` when sites disagree about ownership but not about membership. Listed rather than counted because the dump is otherwise index-only: those xnames appear nowhere else in the payload, so a count would report that a node went missing without saying which one. This matters in practice: `nodes_free` is a conventional pool name (see GUIDE.md), so the same label plausibly exists at several sites. It is a *diagnostic*, not a policy — the conflict-policy open question below is still open, and this endpoint is what makes it observable.
> - **Auth: the management gate**, for a different reason than the refreshes. The dump is the only endpoint that serves group **member lists**, which the security stance below explicitly excludes from the open-by-default set. Operator-only by construction also keeps it clear of any future per-user filtering question — no end user ever sees it.
> - **No filters or pagination.** A hand-called debug endpoint on an Alps-scale site is a few MB, dominated by the xname map and the member lists. The `discarded_members` lists ride on top of that, bounded by how many labels are actually contested — negligible in practice (two 5000-node sites contesting `nodes_free` adds ~75 KB), though a deployment where *most* labels collide would roughly double the payload. `?site=` / summary modes can be added if that ever bites.

The originally-drafted **site CRUD** (`POST /sites`, `PUT /sites/{name}`, `DELETE /sites/{name}`, with mutations persisted back to the config file) is **descoped to an open question** rather than implemented: in the deployment the standalone-service decision implies, configuration is declarative and often a read-only mount, and a service writing service-account credentials back into its own TOML is the part of this draft most likely to be wrong. Revisit with the maintainer if runtime site management is needed at all.

**Integration — decided (2026-07, with the maintainer): CLI-side pre-resolution.** The pre-Stage-3 draft had `manta-server` consult the cache when `X-Manta-Site` was absent, but that founders on per-site tokens: a CSM token is issued by one site's Keycloak, and the CLI chooses which cached token to attach *based on the site* — the very thing being resolved. Even a perfect server-side resolution would see the request arrive bearing the wrong site's credential. Instead, the **`manta` CLI consults the cache service before dispatch**:

1. Explicit `--site` / `cli.toml` `site` — honored as today; the cache is not consulted.
2. The command names a group → `GET /api/v1/lookup/group/{label}` on the cache → site.
3. The command names xnames → `GET /api/v1/lookup/nodes?xnames=…` → the unanimous site, or a clear client-side error listing the per-xname resolutions when the list splits across sites or contains unknowns.
4. Neither → error as today (site is required).

From the resolved site onward everything is unchanged: the right per-site token cache, the right `X-Manta-Site` header, per-user authorisation in the `manta-server` handler. The CLI grows two `cli.toml` keys (`cache_url`, optional `cache_api_token`) and degrades gracefully — cache unreachable means "site required", exactly today's behaviour. `manta-server` needs **no change** for this design.

> **✅ CLI-side pre-resolution delivered** (`manta-cli`'s `common/site_resolution.rs`, hooked into `run_cli` before `AppContext` is built). Scope notes:
>
> - The clap tree has no uniform target-argument id, so the resolver uses a **closed per-command table**: `power on/off/reset group|nodes`, `get nodes`, `get group-nodes`, `get sessions` (via its `--group`/`--xnames` filters), `apply boot group|nodes`, `apply boot-parameters`, `apply kernel-parameters`, `console node`. Commands outside the table simply follow the old no-site path; extend the table in `extract_target` as commands earn support.
> - Only **plain xname lists** resolve. Hostlist bracket expressions and NIDs are skipped (the cache indexes xnames; expansion is server-side), falling back to requiring `--site`.
> - Failure semantics as designed: transport failures warn and degrade to the lazy "No site selected" error; definitive answers (unknown group, split list, rejected `cache_api_token`) abort with a specific message — silently guessing a site could aim a destructive command at the wrong cluster. A successful resolution prints a one-line notice on stderr.
> - Covered by unit tests (target extraction incl. bracket/NID guards, reply interpretation) and three end-to-end `assert_cmd` tests driving the real binary against a mock cache (resolve, degrade, split).

**Security stance (decided 2026-07).** The lookup endpoints serve read-only routing metadata — site names, group labels, xnames; no member lists, credentials, or per-node state — considered non-sensitive inside the deployment perimeter. They are open by default, gated by the optional shared `api_token`, and protected primarily by network placement. Three deliberate caveats: the management endpoints always require the token (above); `GET /api/v1/dump` is management-gated **because** it serves the group member lists this stance excludes, so the "no member lists" justification stays literally true of everything open; and TLS on the cache matters for **integrity** more than secrecy — a tampered resolution answer could steer a destructive command at the wrong cluster, so production deployments should still terminate the cache behind TLS.

**Lifecycle.** (The pre-Stage-3 draft had `manta-server` instantiating the cache in-process; superseded.) The standalone `manta-cache-server` owns its own lifecycle — initial refresh before its listener binds, optional periodic refresh, on-demand management refresh. `manta-server` is not involved; the CLI treats the cache as an optional accelerator and falls back to explicit `--site` when it is absent.

**Cross-cutting concerns** decided in this stage:

- **Conflict policy** when a group label or xname appears at more than one site (reject, prefer a default site, or return all candidates and let the caller disambiguate). Still open. The built `Index` still does not retain that a collision happened, so a *policy* that surfaces candidates needs library support; what exists today is observability only — `GET /api/v1/dump`'s `conflicts` section re-scans the snapshots and names contested labels, while the routing answer remains the silent alphabetical winner.
- **TTL / freshness**. The force half exists (`POST /refresh/{site}` re-syncs a known-mutated site without waiting for the timer); an optional per-site stale window remains open.
- **Persistence**. Decide whether to persist the index to disk (sqlite / JSON snapshot) so restarts skip the cross-site fan-out. May be in scope for Stage 4 or punted to a follow-up.

**Acceptance.** An operator can run a `manta` command that names only a group or only a node list, with no `--site`, and it reaches the right cluster (via the CLI-side pre-resolution above). Failure modes (cross-site xname list, unknown group) produce clear errors naming the per-xname resolutions. The refresh half of runtime management is already delivered; site CRUD is not part of the acceptance unless the open question above resolves in its favour.

## Stage 5 — Production credentials (service accounts)

Stages 3 and 4 were developed and demoed against a **personal** credential: `token_file` pointed at the CLI's own token cache (`<cache_dir>/<site>_auth`), which made the live test a one-liner — whenever the operator re-authenticated, the cache followed. That is not a production deployment. A shared service must not refresh as a named human: the index silently inherits that person's group roles, dies when they leave or their token expires, and attributes every backend call to them.

This stage moves the refresh onto **one Keycloak service account per site**, and — because the two credential kinds are byte-for-byte indistinguishable to the code that uses them — makes the difference *observable* instead of enforced.

### What a service-account credential actually is (verified 2026-08-06)

The credential is the same thing the CLI already accepts in `MANTA_CSM_TOKEN`: a Keycloak-issued JWT used verbatim as `Authorization: Bearer`. There is **no** exchange step, no client-credentials grant, and no Keycloak client for the cache to talk to. Decoding a real prealps service-account token gave:

| Claim | Value |
|---|---|
| `typ` | `Bearer` |
| `azp` | `manta-cache-test` (the Keycloak client id) |
| `preferred_username` | `service-account-manta-cache-test` |
| `iat` → `exp` | 2026-08-05 → 2027-08-05 (**one year**, via the `offline_access` scope) |
| `realm_access.roles` | `default-roles-shasta`, `meda`, `offline_access`, `uma_authorization`, `gallina` |

Three consequences drove the design:

1. **It must be a JWT, not an opaque key.** `manta-cli`'s `SessionContext::from_parts` decodes `preferred_username`, `name`, and `realm_access.roles` from the bearer for every authenticated verb, so an opaque API key would never have worked through the CLI path either. The cache itself does not require this — it only forwards the bearer — but the credential is a JWT in practice, which is what makes the advisory below possible.
2. **Service accounts are exactly identifiable.** Keycloak's convention is `preferred_username == "service-account-" + azp`. That is a string equality, not a heuristic: a human token carries a login name as `preferred_username` and can never satisfy it. So "is this a personal token?" is answerable with certainty.
3. **The token's roles bound what the cache can index.** csm-rs's `get_group_name_available` (`hsm/group/utils.rs`) branches on the JWT: with `pa_admin` it returns every HSM group at the site; without it, the *available groups are the roles themselves*, minus Keycloak internals (`default-roles-shasta`, `offline_access`, `uma_authorization`) and site-wide umbrella names. The prealps account above therefore indexes exactly `meda` and `gallina`.

### Decisions

- **Storage — `token_file` only.** Inline `token` is removed from `[sites.<name>]`; a real Keycloak credential must not live in a config file that gets templated, backed up, or committed. `token_file` is already re-read on every refresh, so annual rotation is a file write with no restart. It is also the one form every deployment target can produce: a Kubernetes projected `Secret`, a systemd `LoadCredential=`, or a bind-mounted file all hand the process a path. `[server] api_token_file` is added alongside for the cache's own inbound bearer, which is the same class of secret.
- **Permissions — warn, never refuse.** A group- or world-readable token file is worth flagging, but Kubernetes projected secrets default to `0644`, so failing closed would break the likeliest deployment target for a condition the operator often cannot change.
- **Coverage — a deployment choice, not a code one.** Whether the service account carries `pa_admin` (cache resolves the whole site) or a scoped role set (cache resolves only those groups, everything else needs an explicit `--site`) is a Keycloak-side decision. The code is indifferent, and the CLI already degrades correctly: an unresolvable group falls back to "site is required", per the Stage-4 accelerator-not-dependency stance. `is_admin` is surfaced on `/dump` so the choice in force is visible without a role audit.
- **Keep both credential kinds working, warn on personal ones.** No enforcement, no `--allow-user-token` escape hatch. The mechanism stays token-agnostic — which keeps [LIVE-TEST.md](LIVE-TEST.md)'s point-at-your-own-token flow working for demos — while the advisory makes a misdeployed personal token impossible to miss in the startup summary and on `/dump`.
- **An empty snapshot is a failure, not a success.** See below.

### The credential advisory

On every token read the server decodes the JWT payload locally — base64 + JSON, no signature verification, exactly as `manta_shared::common::jwt_ops` does — and derives `principal` (`azp` for a service account, `preferred_username` for a person), `kind` (`service_account` / `user` / `unknown`), `expires_at`, `expires_in_days`, and `is_admin`. This is surfaced in two places: the startup summary, and a `credential` block on each `/api/v1/dump` site entry, next to the `token_file` path. The token value itself is never rendered anywhere.

Warnings fire when the credential is a personal token, when it expires within 30 days, and when it has already expired (at `ERROR` level — a lapsed credential is an outage, not a heads-up). The near-expiry warning is the one that earns its keep: a credential that works for twelve months and then stops is precisely the kind nobody remembers, and `expires_at` on `/dump` gives monitoring something to alert on.

Two details that look like nits and are not:

- **`expires_in_days` floors rather than truncating.** `TimeDelta::num_days` divides toward zero, so a credential six hours past `exp` reports `0` — indistinguishable from one lapsing six hours from now, and enough to make the obvious `expires_in_days < 0` alert miss the entire first day of an outage. The warning text classifies on the timestamp directly, so it can never describe a lapsed credential as "expires in 0 day(s) — replace it before it lapses".
- **The log dedup compares against the warnings previously *logged*, not against `previous.warnings(now)`.** Re-deriving both sides at the same instant makes them equal by construction for an unchanged token file, which suppresses every message after the first — a server booted eleven months before expiry would never report the lapse. The stored-warnings comparison is what makes the warning appear on the day it becomes true. `credential_state_changed` is a pure predicate precisely so this is testable; the regression test asserts on that decision, since the warnings are returned to the caller either way.

The decode lives in `manta-cache-server`, not in `manta-cache` and not via `manta-shared`. Reaching for `manta_shared::common::jwt_ops` would drag `manta-backend-dispatcher` (+ csm/ochami types) into a deliberately standalone binary — the same trade already refused twice, for `config.rs`'s `ProjectDirs` lookup and `refresh.rs`'s `GroupNode` mirror. Thirty lines and a `base64` dependency is the cheaper side of that bargain.

### Empty snapshots

A site whose `GET /groups/available` returns `200 []` used to count as a successful refresh: `/dump` showed `in_index: true`, a fresh `refreshed_at`, and `last_error: null` — a site that looks perfectly healthy and resolves nothing, with no cause recorded anywhere.

Partial visibility is expected (it is just the account's role set), but *empty* is not: there is no configuration in which you would list a site in `cache-server.toml` and want it to contribute nothing. It means the credential resolved to no HSM roles at all — expired, wrong account, roles removed, or pointed at the wrong site. A zero-label snapshot is therefore converted into a per-site failure carrying a diagnostic message.

The conversion lives in `manta-cache-server::refresh`, not the library: `manta-cache` deliberately stays policy-free and hands availability decisions to the caller via `RefreshOutcome`. Both refresh paths then reuse the machinery already in place — `refresh_site` keeps the previous snapshot serving and records `last_error`, while `refresh_all` drops the site with `refreshed_at: null`, matching its documented wholesale-replace contract. Preserving a stale-but-good snapshot through a credential outage was considered and rejected as encroaching on the still-open stale-window question.

**Acceptance.** `cache-server.toml` rejects an inline site token rather than ignoring it (the schema is `deny_unknown_fields`, so a half-migrated config that leaves `token` beside `token_file` fails to load instead of quietly parking a live credential in the file — and the error carries the migration); a cache refreshing as a personal token says so loudly at startup and on `/dump`; each site's principal, expiry and admin status are visible without decoding anything by hand; and a site that returns no groups is reported with a cause instead of appearing healthy.

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

**Step-by-step runbook: [LIVE-TEST.md](LIVE-TEST.md)** (three-terminal setup, config snippets, and the walkthrough below as concrete commands; verified 2026-07-19 against prealps).

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
| ~~Service-account token sourcing (config / env / Vault) + rotation~~ | ~~Stage 3~~ | **Resolved (2026-08): `token_file` only**, one Keycloak service account per site; inline `token` removed. The file is re-read every refresh, so rotation needs no restart, and every deployment target (k8s projected Secret, systemd `LoadCredential=`, bind mount) can produce a path. See Stage 5. |
| Service-account group coverage: `pa_admin` vs scoped roles | Deployment-time, per site | Not a code decision — csm-rs derives available groups from the token's realm roles, so the account's roles bound what the cache can index. Scoped is safe (unresolvable groups fall back to explicit `--site`); `pa_admin` makes the cache resolve the whole site. `/dump`'s `credential.is_admin` shows which is in force. |
| Conflict policy when label / xname spans sites | Stage 4 | Only matters once an integration layer needs to *resolve* something to a single site. Stage 1 ships documented deterministic rules (see `Index`'s "Conflict handling" doc), not a policy — and the built `Index` does not retain that a collision happened, so surfacing candidates *in a lookup answer* needs library support. `GET /api/v1/dump` makes collisions observable in the meantime by re-scanning the snapshots. |
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
