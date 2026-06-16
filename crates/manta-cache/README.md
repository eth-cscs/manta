# manta-cache

> **Status:** planning. This crate **does not exist yet** — the source tree under `crates/manta-cache/` is a placeholder holding this README and [ROADMAP.md](ROADMAP.md). The first cut of the cache lives as a module inside `manta-server` (or `manta-shared`); the crate is created when that module is extracted at Stage 2 of the roadmap.

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

Both indexes are populated by walking the existing per-site `GET /api/v1/groups/available` and `GET /api/v1/groups/nodes` endpoints on each manta-server.

The cache holds **routing information only**. It does not duplicate per-node state (power status, boot parameters, CFS components, IMS images), and it is not a replacement for HSM — the canonical group membership lives in CSM / OpenCHAMI.

---

## Not in scope

- Caching per-node state — power, boot params, CFS components, IMS metadata. The cache resolves **routing**, not cluster state.
- Replacing `[sites.*]` in `server.toml`. The cache reads from a sibling config that lists *which manta-servers exist*; per-site backend connection details (CSM/OCHAMI URLs, Vault, k8s) stay where they are.
- Per-user authorisation. The cache is a routing layer; the existing per-user authorisation continues to run in the `manta-server` handler that receives the resolved request.

---

## Implementation plan

The roadmap is staged: a Rust module inside an existing crate first, then an extraction into the `manta-cache` crate, then an HTTP wrapper, then management functionality and `manta-server` integration. See [ROADMAP.md](ROADMAP.md) for the per-stage detail, acceptance criteria, and open questions.
