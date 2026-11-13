# manta-cache

> **Status:** Stages 1–3 landed. This crate is the **core library** (built directly as a standalone workspace member, collapsing the roadmap's Stage 1 + Stage 2): `SiteDescriptor`, `Index` with synchronous lookups (`group_to_site`, `xname_to_site`, `groups`, `sites`, `group_members`), an async partial-failure-tolerant `refresh()`, and an HTTP-free constructor (`Index::from_snapshots`). The Stage-3 HTTP API ships as the sibling **`crates/manta-cache-server`** binary — a standalone shared lookup service (see the deployment-shape decision in [ROADMAP.md](ROADMAP.md)). `manta-server` consults it (Stage 4): a request that carries no `X-Manta-Site` but names a group or plain xname list is resolved through the cache when `server.toml` configures `[site_resolution]` (see the root README and API.md § Site resolution).

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

manta-cache is the component that maintains that mapping and exposes lookups against it. `manta-server` is the one that asks: the CLI simply leaves the site out, and the server looks it up from the group or node list in the request.

## What's actually cached

Conceptually a list of `(site_name, group_label, member_xnames…)` triples, plus two derived lookup indexes:

- **`group_label → site_name`** — for commands that target a group directly.
- **`xname → site_name`** (via the group-membership join) — for commands that target a list of nodes.

Both indexes are populated from one existing endpoint per site: `GET /v2/groups` on the manta-server, which lists every group the cache's service account can access, with its members. The cache holds exactly those groups; a group the account cannot access is not indexed, even when one of its nodes is.

The cache holds **routing information only**. It does not duplicate per-node state (power status, boot parameters, CFS components, IMS images), and it is not a replacement for HSM — the canonical group membership lives in CSM / OpenCHAMI.

---

## Production credentials

The cache refreshes each site by calling that site's `manta-server` with a bearer token, exactly like a `manta` client. manta-server exchanges the token on every request for one the site accepts (`[sites.X.token_exchange]` in its `server.toml`), so it must be a **CSCS Keycloak** token, and one token is valid for every site.

### A CSCS Keycloak service account, a new token per request

The cache refreshes as a **CSCS Keycloak service account** (a confidential client of the CSCS realm), with the same variables the CLI reads in CI:

```bash
export MANTA_SA_CLIENT_ID=…       # the service account's client id
export MANTA_SA_CLIENT_SECRET=…   # its client secret
manta-cache-server
```

Both are required; the server refuses to start without them. It gets a **new token for every request** to manta-server (one per site per refresh) with the client credentials grant and caches nothing. A stored token would not do: an OpenCHAMI site's Hydra accepts each token only once, and CSCS access tokens last minutes. At startup the server makes one grant before anything else, so wrong credentials stop it with the Keycloak's reason instead of showing up as every site failing.

The secret lives only in the environment, never in `cache-server.toml`. The config sets at most the issuer, which defaults to the production CSCS realm, as for the CLI:

```toml
[auth]
issuer_url = "https://auth.cscs.ch/auth/realms/cscs"   # the default; override only for testing

[manta_server]
url   = "https://manta-server.example.ch:8443"
sites = ["alps", "prealps"]
```

`[manta_server]` names the **one** manta-server the cache indexes and answers for — the one whose `[site_resolution]` points at the cache — and which of its sites to index. One server rather than one per site: a site that server does not host would be an answer it cannot use.

The startup summary and `GET /api/v1/dump` (`service_account`) show the client id and issuer, never the secret. A shared service should not refresh as a named human: it would inherit that person's group roles and attribute every backend call to them. Since the credentials are a client id and secret, that cannot happen by accident.

### How much of a site the cache can see

The site derives the available groups from the roles of the token manta-server obtains by exchanging the service account's CSCS token there, so **the service account's roles, as each site maps them, bound what the cache can index**:

| Account holds | Cache indexes | Effect on users |
|---|---|---|
| admin at the site | every HSM group at the site | any group resolves without `--site` |
| specific group roles | only those groups | other groups need an explicit `--site` |
| no group roles | nothing | the site is reported as failed (see below) |

Both of the first two are legitimate; which you want is a deployment decision. Scoped is the safer default: a group the cache does not know fails with "pass --site" rather than resolving to a guess. Check the result in the indexed group count on `GET /api/v1/dump`.

A site that returns **no** groups is treated as a failed refresh, not a healthy empty one: it drops out of the index with an explanatory `last_error` rather than appearing fresh and resolving nothing. A token the Keycloak refuses fails every site the same way, with `no token for site '<name>': …` as the error, and nothing is sent to manta-server.

### Rotation

Rotating the secret means restarting with the new value in the environment. Nothing else holds it.

The cache's own inbound bearer (`[server] api_token`) can be kept out of the config file with `api_token_file`. It is read once at startup, so rotating it needs a restart; a file readable beyond its owner draws a startup warning (never a refusal: Kubernetes projected `Secret` volumes default to `0644`).

---

## Not in scope

- Caching per-node state — power, boot params, CFS components, IMS metadata. The cache resolves **routing**, not cluster state.
- Replacing `[sites.*]` in `server.toml`. The cache's own config names the manta-server and the sites to index; per-site backend connection details (CSM/OCHAMI URLs, Vault, k8s) stay where they are.
- Per-user authorisation. The cache is a routing layer; the existing per-user authorisation continues to run in the `manta-server` handler that receives the resolved request.

---

## Implementation plan

The roadmap is staged: a Rust module inside an existing crate first, then an extraction into the `manta-cache` crate, then an HTTP wrapper, then management functionality and `manta-server` integration. See [ROADMAP.md](ROADMAP.md) for the per-stage detail, acceptance criteria, and open questions.
