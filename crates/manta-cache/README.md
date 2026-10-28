# manta-cache

> **Status:** Stages 1–3 landed. This crate is the **core library** (built directly as a standalone workspace member, collapsing the roadmap's Stage 1 + Stage 2): `SiteDescriptor`, `Index` with synchronous lookups (`group_to_site`, `xname_to_site`, `groups`, `sites`, `group_members`), an async partial-failure-tolerant `refresh()`, and an HTTP-free constructor (`Index::from_snapshots`). The Stage-3 HTTP API ships as the sibling **`crates/manta-cache-server`** binary — a standalone shared lookup service (see the deployment-shape decision in [ROADMAP.md](ROADMAP.md)). The `manta-server` integration (Stage 4) is not built yet.

A site-resolution cache for manta. It learns which **site** (CSM / OpenCHAMI cluster) each group and each compute node lives at, so the user does not have to name the site explicitly on every command.

This README is the plain-English explanation of *what* the cache is and *why* it exists, aimed at anyone landing on the crate cold — including readers with no prior manta, CSM, or OpenCHAMI background. For the staged implementation plan, acceptance criteria, and open questions, see [ROADMAP.md](ROADMAP.md).

---

## What manta is

**manta** is a tool for running operations on HPC compute clusters — power nodes on or off, change what kernel they boot, deploy new node images, open serial consoles, and so on. It ships as two binaries:

- `manta` — a command-line client that operators run on their workstation.
- `manta-server` — an HTTPS server that holds the credentials and per-cluster connection details. The CLI never talks to the underlying cluster directly; it goes through `manta-server`.

The cluster being managed runs one of two control planes:

- **CSM** — Cray System Management. Hewlett Packard Enterprise's stack for managing the Cray EX line of supercomputers; provides REST APIs for boot, configuration, sessions, and hardware state.
- **OpenCHAMI** — an open-source re-implementation of much of the same surface, designed for sites that want to run CSM-shaped clusters without the HPE stack.

`manta-server` speaks to one or the other through a backend layer; both expose the same logical concepts (groups, sessions, images, configurations) even if the underlying APIs differ.

## What a "site" is

A **site**, in manta's vocabulary, is one CSM or OpenCHAMI deployment. An organisation that operates several clusters has several sites: `alps`, `prod-b`, `lab-test`, and so on. Each site has its own URL, its own credentials, and (usually) its own physical machines.

`manta-server` knows about every site that its operator configured in `server.toml`. The CLI, on the other hand, must say *which* site each command targets — either via `site = "<name>"` in `cli.toml`, or `--site <name>` on the command line.

## What HSM groups are

Each site organises its nodes into **HSM groups** — named buckets of node identifiers (e.g. `compute`, `gpu-cluster`, `nodes_free`). A node belongs to a group iff it appears on that group's member list. A node can belong to several groups at once, and groups are the primary unit of targeting for almost every manta command (`manta power off group compute`, `manta apply boot group gpu-cluster`, etc.).

For a longer explanation, see [GUIDE.md §2 — Groups](https://github.com/eth-cscs/manta/blob/main/GUIDE.md#2-groups).

## Why a cache

Today every `manta` invocation must know which site to talk to. The operator either sets it once in `cli.toml` or names it per-command with `--site`. For an operator who works against a single site this is harmless, but for one juggling several it has two costs:

1. **Friction.** Before issuing any command, the operator has to remember which site a group or node belongs to.
2. **Risk.** A wrong `--site` either errors out (best case) or — worse — runs the command against the wrong cluster.

Most of this friction is structurally unnecessary. Group membership at each site changes infrequently, so the mapping `(site, group, members)` can be cached. Once it is:

- `manta power on group gpu-cluster` resolves the site from the group label alone.
- `manta power reset nodes x3000c0s1b0n0` resolves the site from the node's group membership.

manta-cache is the component that maintains that mapping and exposes lookups against it.

## What's actually cached

Conceptually a list of `(site_name, group_label, member_xnames…)` triples, plus two derived lookup indexes:

- **`group_label → site_name`** — for commands that target a group directly.
- **`xname → site_name`** (via the group-membership join) — for commands that target a list of nodes.

Both indexes are populated by walking the existing per-site `GET /v2/groups/available` and `GET /v2/groups/nodes` endpoints on each manta-server.

The cache holds **routing information only**. It does not duplicate per-node state (power status, boot parameters, CFS components, IMS images), and it is not a replacement for HSM — the canonical group membership lives in CSM / OpenCHAMI.

---

## Production credentials

The cache refreshes each site by calling that site's `manta-server` with a bearer token, configured per site in `cache-server.toml`:

```toml
[sites.alps]
manta_server_url = "https://manta-server.example.ch:8443"
token_file = "/run/secrets/manta-cache/alps-token"
```

`token_file` is the **only** form — there is deliberately no inline `token`. A Keycloak credential does not belong in a config file that gets templated, backed up, or committed, and a path is the one thing every deployment target can produce: a Kubernetes projected `Secret`, a systemd `LoadCredential=` (read it from `$CREDENTIALS_DIRECTORY`), or a plain bind mount.

### Use a service account, one per site

The token must be a **Keycloak service account**, not a person's login. A shared service refreshing as a named human inherits that person's group roles, stops working when their token expires or their account changes, and attributes every backend call to them.

Create one service account per site and write its token into the site's `token_file`. Nothing enforces this — pointing `token_file` at a personal token still works, which is what keeps the [LIVE-TEST.md](LIVE-TEST.md) demo flow usable — but the server says so loudly, in the startup summary and under `credential.warnings` in `GET /api/v1/dump`:

```
[site: alps]
  manta_server_url: https://manta-server.example.ch:8443
  token_file:       /run/secrets/manta-cache/alps-token
  credential:       service account 'manta-cache-alps', scoped to its own roles, expires 2027-08-05 (364 days)
```

### How much of a site the cache can see

csm-rs derives the available groups from the token's own Keycloak realm roles, so **the service account's roles bound what the cache can index**:

| Account holds | Cache indexes | Effect on users |
|---|---|---|
| `pa_admin` | every HSM group at the site | any group resolves without `--site` |
| specific group roles | only those groups | other groups need an explicit `--site` |
| no group roles | nothing | the site is reported as failed (see below) |

Both of the first two are legitimate; which you want is a deployment decision. Scoped is the safer default and degrades gracefully — the CLI falls back to "site is required" for anything unresolvable, since the cache is an accelerator rather than a dependency. `credential.is_admin` on `GET /api/v1/dump` shows which is in force.

A site that returns **no** groups is treated as a failed refresh, not a healthy empty one: it drops out of the index with an explanatory `last_error` rather than appearing fresh and resolving nothing.

### Rotation and expiry

The token file is re-read on **every** refresh, so rotation is a file write — no restart, no request drop. CSCS service-account tokens are minted for a year, which is exactly long enough to forget, so the cache surfaces the deadline in three places:

- the startup summary line above,
- `credential.expires_at` and `credential.expires_in_days` on `GET /api/v1/dump` — the field to point monitoring at. It counts down in whole days and goes **negative** once the credential has lapsed, so `expires_in_days < 0` is a correct alert from the first hour of an outage,
- a `WARN` log line once fewer than 30 days remain, escalating to `ERROR` once it has lapsed. These are logged when the state first changes rather than on every refresh, so a month-long warning does not bury the log.

### File permissions

Secret files readable beyond their owner draw a startup warning naming the file and its mode. It is only ever a warning: Kubernetes projected `Secret` volumes default to `0644`, so refusing to start would break the likeliest deployment target over a mode the operator often cannot change. Where you do control it, `0600` is right.

The cache's own inbound bearer (`[server] api_token`) can be kept out of the config file the same way, with `api_token_file`. Unlike the per-site tokens it is read once at startup, so rotating it needs a restart.

---

## Not in scope

- Caching per-node state — power, boot params, CFS components, IMS metadata. The cache resolves **routing**, not cluster state.
- Replacing `[sites.*]` in `server.toml`. The cache reads from a sibling config that lists *which manta-servers exist*; per-site backend connection details (CSM/OCHAMI URLs, Vault, k8s) stay where they are.
- Per-user authorisation. The cache is a routing layer; the existing per-user authorisation continues to run in the `manta-server` handler that receives the resolved request.

---

## Implementation plan

The roadmap is staged: a Rust module inside an existing crate first, then an extraction into the `manta-cache` crate, then an HTTP wrapper, then management functionality and `manta-server` integration. See [ROADMAP.md](ROADMAP.md) for the per-stage detail, acceptance criteria, and open questions.
