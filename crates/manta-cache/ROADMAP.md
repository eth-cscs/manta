# manta-cache — roadmap

> **Status:** planning. This crate **does not exist yet** — the source tree under `crates/manta-cache/` is a placeholder. The first cut of the cache lives as a module inside `manta-server` (or `manta-shared`); the crate is created when that module is extracted at Stage 2.

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
- Unit tests that exercise the lookup methods against fixture inputs (no live `manta-server` needed).
- A single integration test that runs `refresh` against one live `manta-server` and asserts the index shape — gated behind an env var so CI without a backend skips it.

No public API stability promise — the module is internal-only. Other code in the same crate may import it; nothing outside the crate sees it.

**Acceptance.** Module compiles, unit tests pass, the integration test passes against a known-good `manta-server` URL + token.

## Stage 2 — Extract into the `manta-cache` crate

Promote the Stage-1 module into an independent workspace member at `crates/manta-cache/`. **This is the first time the crate physically exists**; until this stage lands, the directory only carries this roadmap and the sibling README.

Steps:

- `git mv` the Stage-1 module's source files from `manta-server/src/cache/` (or wherever Stage 1 placed them) into `crates/manta-cache/src/`.
- Add `crates/manta-cache/Cargo.toml` with the workspace-inherited metadata pattern the other crates use (`version.workspace = true`, etc.).
- Register the crate in the workspace `Cargo.toml`'s `members` array.
- Define the **public** API surface: the same lookup + refresh methods Stage 1 already shipped, but now exposed via `pub` and documented in the crate root.
- Update `manta-server` to depend on `manta-cache = { workspace = true }` and switch its call sites from the in-place module to the crate's public API.

**Acceptance.** `cargo build -p manta-cache` succeeds standalone, `manta-server` builds against it, and the existing `manta-server` behaviour is unchanged (the cache is still in-process; only its source-tree location moved).

## Stage 3 — HTTP APIs

Wrap the crate in a small HTTP service so the cache can be queried **from outside its host process**. This is what makes "one cache shared by several `manta-server` instances" possible — until now every `manta-server` would have to build and hold its own copy.

Endpoints, draft (final shapes TBD during implementation):

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/sites` | List cached sites |
| `GET` | `/lookup/group/{label}` | Resolve `group → site` |
| `GET` | `/lookup/nodes?xnames=…` | Resolve a comma-separated xname list → site(s) |

**Deployment shape is decided at this stage**, not before. Three viable shapes:

1. **In-process endpoints on `manta-server`** — the simplest; the cache is just more routes on the existing Axum router. Each `manta-server` still owns its own cache.
2. **Standalone shared service** — a separate `manta-cache-server` binary that all `manta-server` instances point at. One cache for the whole org. More ops cost; biggest payoff.
3. **Sidecar binary** — one `manta-cache-server` colocated 1:1 with each `manta-server`, talked to over a Unix socket or loopback HTTP. Process isolation without a network hop.

**Authentication** also lands here. The cache has to call `GET /groups/*` on every site to refresh, which means it needs a bearer token per site. Two models:

- **Service-account-style token per site** (the default expectation). The cache holds one rotated, scoped token per site; all users share the resulting index; per-user authorisation continues to run in the `manta-server` handler that ultimately receives the resolved request.
- **Per-user cache** — the index is built using the caller's own token and partitioned per `(user, site)`. Better authorisation fidelity at cache level, but multiplies traffic and memory by the user count.

Both decisions are noted as [Open questions](#open-questions); the implementer of Stage 3 picks before writing code.

**Acceptance.** A `curl` from outside the host process can query the three endpoints; the shape and content of the responses match what the in-process API returned at Stage 2.

## Stage 4 — Management functionalities + manta-server integration

The final stage delivers the user-visible payoff and the operability surface needed to run the cache in production.

**Management endpoints.** Round out the API so the cache can be operated without restarts:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/refresh` | Full re-sync of every site |
| `POST` | `/refresh/{site}` | Re-sync one site |
| `POST` | `/sites` | Add a new site |
| `PUT` | `/sites/{name}` | Update one site (URL, token, …) |
| `DELETE` | `/sites/{name}` | Drop a site |

Site mutation persists back to the cache's config file so a restart does not lose the change.

**manta-server integration.** Wire `manta-server` to consult the cache when an incoming request arrives **without** an explicit `X-Manta-Site` header but **with** group- or node-level targeting. Resolution order:

1. Explicit `X-Manta-Site` header — honored as today; cache is bypassed.
2. Request body / query carries a group label → cache `group → site` lookup → set the effective site for the rest of the handler.
3. Request body / query carries xnames → cache `xname → site` lookup. If every xname resolves to the same site, use it. If they split across sites, return `400 Bad Request` with a body listing the conflicting `(xname, site)` pairs.
4. None of the above → `400` as today (site is required).

The CLI side then becomes: `manta` keeps the `site = "<name>"` default in `cli.toml` for convenience but drops the *requirement* entirely for any command that already names a group or node list.

**Cross-cutting concerns** decided in this stage:

- **Conflict policy** when a group label or xname appears at more than one site (reject, prefer a default site, or return all candidates and let the caller disambiguate).
- **TTL / freshness**. Optional per-site stale window; admin force-stale endpoint so a known-mutated site can be re-synced without waiting for the timer.
- **Persistence**. Decide whether to persist the index to disk (sqlite / JSON snapshot) so restarts skip the cross-site fan-out. May be in scope for Stage 4 or punted to a follow-up.

**Acceptance.** An operator can run a `manta` command that names only a group or only a node list, with no `--site`, and it reaches the right cluster. The cache's state can be mutated at runtime through the API. Failure modes (cross-site xname list, unknown group) produce clear `400` responses.

---

## Open questions

These are the decisions the roadmap deliberately punts on until the stage that actually needs them. Listed once here so they are easy to find and revisit.

| Question | Decide at | Why deferred |
|---|---|---|
| `manta-server` vs `manta-shared` as Stage-1 home | Stage 1 kickoff | Both are technically viable; depends on whether `manta-cli` is ever expected to consume the cache. |
| Deployment shape: in-proc / sidecar / standalone | Stage 3 | The choice does not affect the Stage-1 or Stage-2 code; only the wrapper around it. |
| Auth model: service-account vs per-user | Stage 3 | Same — Stages 1 and 2 take a token, they don't care where it came from. |
| Conflict policy when label / xname spans sites | Stage 4 | Only matters once an integration layer needs to *resolve* something to a single site. |
| Persistence (in-memory vs on-disk snapshot) | Stage 4 | A cold start is a few HTTP calls per site; tolerable until the deployment shape pushes back. |
| Refresh cadence (pull-on-demand vs periodic background) | Stage 4 | Depends on the deployment shape and traffic pattern. |

## What I still need from you

Two short answers will firm up the next two stages:

1. **Stage 1 home — `manta-server` or `manta-shared`?** Defaulting to `manta-server` unless you tell me otherwise.
2. **Anything about Stage 3 that should be locked now rather than deferred?** (Deployment shape and auth model are the big two.) If you have a strong preference, I'll fold it into the Stage-3 section now; otherwise the open-questions table above is the record.
