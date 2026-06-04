# Migrating from manta v1 to v2

This guide covers everything that changed between the v1.x series
(last release: `v1.64.3`) and the v2 series (current: `v2.0.0-beta.12`).
The architectural shape, the on-disk config layout, the CLI surface,
and the HTTP API all changed; the deployment topology gained a second
binary.

The guide is organised by audience:

- [End users](#1-end-users-running-manta-on-a-workstation) running the
  `manta` CLI on a workstation
- [Site operators](#2-site-operators-deploying-the-stack) deploying the
  stack
- [Integrators](#3-integrators-shell-scripts-and-programmatic-http-clients)
  with shell scripts or programmatic HTTP clients

If you skim only one section, make it
[§4 Step-by-step playbook](#4-step-by-step-playbook).

---

## What changed at a high level

| Aspect | v1.x | v2 |
|---|---|---|
| **Binaries** | One — `manta` (CLI talks to CSM/OCHAMI direct) | Two — `manta` (CLI) and `manta-server` (HTTPS API in front of CSM/OCHAMI) |
| **Auth target** | Each user's CLI authenticates to the backend directly | CLI authenticates to `manta-server`; the server holds the backend creds and tokens |
| **Config file** | Single `~/.config/manta/config.toml` mixing CLI + backend + (in some setups) server fields | Split into `cli.toml` (workstation) and `server.toml` (server host); the CLI strictly does not need backend URLs |
| **CLI verbs** | `apply session`, `apply boot cluster`, `migrate vCluster backup`, `get cluster`, `get hardware cluster`, `power on/off/reset cluster`, `add-nodes-to-groups`, `remove-nodes-from-groups`, `apply hardware cluster`, `update boot-parameters`, `update redfish-endpoints`, `config gen-autocomplete`, … | Renamed and the old forms removed: `run session`, `apply boot group`, `backup vcluster`, `get group-nodes`, `get group-hardware`, `power on/off/reset group`, `add nodes`, `delete nodes`, `apply hardware group`, `apply boot-parameters`, `apply redfish-endpoint`, `gen-autocomplete`, … The full mapping is in §1.4 below |
| **CLI flags** | `--hsm-group`, `--target-cluster`, `--parent-cluster`, `--create-hsm-group`, … | `--group`, `--target-group`, `--parent-group`, `--create-group`, … Old flag names retained as visible clap aliases |
| **CLI output** | Mix of plain `println!` and ad-hoc JSON dumps; `--output json` only on some `get` commands | Every mutating command honours `-o/--output {table,json}`; JSON envelope is `{"status":"ok","message":"...","data":...}` |
| **HTTP API** | None — there was no server | Documented REST + WebSocket API on `manta-server`; see [API.md](API.md) |
| **Logging** | Mostly silent | Structured `tracing` throughout — request curl-equivalents at DEBUG, auth chain summary at INFO; server prints loaded config + per-site backend URLs on startup |
| **Tests / CI** | — | 376 workspace tests, clippy-clean with `-D warnings`, pinned Rust toolchain (`rust-toolchain.toml`), per-crate Dockerfiles, OpenAPI spec served at `/openapi.json` |

The most important conceptual change: **v1 dispatched directly to the
backend from the CLI; v2 puts an HTTPS server (`manta-server`) between
the user and the backend.** Tokens, vault paths, k8s service-account
credentials, and TLS material all live on the server now, never on
the workstation. Every CLI call goes out as
`HTTPS → manta-server → CSM/OCHAMI`.

---

## 1. End users (running `manta` on a workstation)

### 1.1 Install the new binary

The CLI binary is still called `manta` (the Cargo package is now
`manta-cli`, but the produced executable keeps the same name).
Install it the same way you installed v1 — `cargo install`, the
prebuilt release archive, or your distro package — then drop the v1
binary from `$PATH`.

```bash
# Verify
$ manta --version
manta 2.0.0-beta.12
```

### 1.2 Convert your config file

v1 used a single `~/.config/manta/config.toml`. v2 splits it into
two files:

- `~/.config/manta/cli.toml` — what your workstation needs
- `~/.config/manta/server.toml` — what the server needs (only present
  on the box that runs `manta-server`; workstation users don't need
  this file at all)

The CLI auto-detects an obsolete `config.toml` and points you at
the migration mapping when first run with no `cli.toml` present. The
mapping is:

```
copy these fields verbatim:        log, site, parent_hsm_group, auditor
add CLI-only (now required):       manta_server_url = "https://..."
                                   (CLI talks only to the manta server)
drop (no longer recognised):       audit_file (audit emission is
                                   server-side only)
do not copy (server-only fields):  [sites.*] (every backend connection
                                   bundle lives in server.toml now), the
                                   [server] section, and the old
                                   sites.<X>.manta_server_url key
```

Minimal v2 `cli.toml`:

```toml
log = "info"
site = "alps"
parent_hsm_group = ""
manta_server_url = "https://manta-server.example.com:8443"
```

> The CLI struct has no `[sites]` table. Any `[sites.*]` block left
> over from a v1 `config.toml` is silently ignored by the CLI but
> belongs in your operator's `server.toml` — see [§2 Server setup](#2-server-setup).

> `manta_server_url` is **required**. Ask your site operator for the
> URL — it's whatever they used as `listen_address`/`port` in the
> server's `server.toml`.

### 1.3 Re-authenticate

v1 cached a CSM token directly. v2 caches a token issued by
`manta-server` (which proxies the credential exchange to the
backend). Existing token files in `~/.cache/manta/` should be
cleared:

```bash
manta config unset auth     # interactive picker, removes one token file
# or
rm -rf ~/.cache/manta/
```

The first command you run on v2 will re-prompt for Keycloak
credentials.

### 1.4 Update muscle memory

The verb renames listed below have **already been removed** — these
old forms no longer resolve and you must update every call site
before upgrading. (The `redfish-endpoint` singular noun on
`add` / `delete` is still kept as a visible clap alias on the new
plural; the `apply redfish-endpoint` canonical command uses the
singular spelling.)

| Removed v1 form | Canonical v2 form |
|---|---|
| `manta apply session` | `manta run session` |
| `manta apply boot cluster <N>` | `manta apply boot group <N>` |
| `manta apply hardware cluster` | `manta apply hardware group` |
| `manta get cluster <N>` | `manta get group-nodes <N>` |
| `manta get hardware cluster <N>` | `manta get group-hardware <N>` |
| `manta power on cluster <N>` | `manta power on group <N>` |
| `manta power off cluster <N>` | `manta power off group <N>` |
| `manta power reset cluster <N>` | `manta power reset group <N>` |
| `manta migrate vCluster backup` | `manta backup vcluster` |
| `manta migrate vCluster restore` | `manta restore vcluster` |
| `manta config gen-autocomplete` | `manta gen-autocomplete` |
| `manta update boot-parameters` | `manta apply boot-parameters` |
| `manta update redfish-endpoints` | `manta apply redfish-endpoint` |
| `manta add-nodes-to-groups` | `manta add nodes` |
| `manta remove-nodes-from-groups` | `manta delete nodes` |

Flag renames (all old spellings kept as visible aliases):

| v1 flag | v2 flag |
|---|---|
| `--hsm-group` | `--group` |
| `--target-cluster` | `--target-group` |
| `--parent-cluster` | `--parent-group` |
| `--create-hsm-group` | `--create-group` |
| `--delete-hsm-group` | `--delete-group` |
| `--create-target-hsm-group` | `--create-target-group` |
| `--delete-empty-parent-hsm-group` | `--delete-empty-parent-group` |

The deprecated forms will be **removed in the next major release**
(`v3.0.0`). Use the deprecation grace period to fix your shell
history and any aliases / wrapper scripts.

### 1.5 New things worth knowing

- **`--output json` everywhere.** Every mutating command now returns
  a structured JSON envelope (`{"status":"ok","message":...,"data":...}`)
  when called with `-o json`. Scripts can parse it instead of
  screen-scraping.
- **`config show -o json`** dumps your active settings as a single
  JSON object. Useful for `jq`-driven inventory.
- **`log = "debug"` in `cli.toml`** makes every outbound HTTP call
  print a copy-pasteable `curl` invocation (passwords and tokens
  auto-redacted) — handy when something looks wrong.
- **`manta run session`** is the canonical CFS session create+watch
  workflow (the old `manta apply session` form has been removed).

### 1.6 Regenerate your shell completion

```bash
manta gen-autocomplete --shell zsh --path ~/.zsh/completions
manta gen-autocomplete --shell bash --path /etc/bash_completion.d
manta gen-autocomplete --shell fish --path ~/.config/fish/completions
```

> The legacy `manta config gen-autocomplete` form has been removed —
> use the top-level `manta gen-autocomplete` shown above.

The new completion script reflects the v2 command tree (and the
deprecated forms still autocomplete so old habits work during the
grace period).

---

## 2. Site operators (deploying the stack)

### 2.1 Deploy `manta-server`

v1 had no server-side binary. v2 ships `manta-server` as a separate
executable that fronts the CSM/OCHAMI backends over HTTPS. Plan to
run **one `manta-server` per site**, behind a reverse proxy.

```bash
$ manta-server --version
manta-server 2.0.0-beta.12

$ manta-server --help
Usage: manta-server [OPTIONS]

  --port <PORT>                  Override [server] port from server.toml
  --listen-address <ADDRESS>     Override [server] listen_address
  --cert <CERT>                  Override [server] cert
  --key <KEY>                    Override [server] key
```

A Dockerfile is provided (`crates/manta-server/Dockerfile`).
Systemd-unit example is at the bottom of [README.md](README.md).

### 2.2 Author `server.toml`

Location is `~/.config/manta/server.toml` by default, or set
`$MANTA_SERVER_CONFIG=/path/to/server.toml`. The server logs the
resolved path on startup so you can confirm.

Minimal `server.toml`:

```toml
log = "info"

[server]
listen_address = "0.0.0.0"
port = 8443
cert = "/etc/manta/server.crt"
key = "/etc/manta/server.key"
console_inactivity_timeout_secs = 1800
auth_rate_limit_per_minute = 60   # per source IP for /auth/*; omit to disable

[sites.alps]
backend = "csm"                   # or "ochami"
shasta_base_url = "https://api.cscs.ch"
root_ca_cert_file = "/etc/manta/alps_root_cert.pem"

[sites.alps.k8s]
api_url = "https://10.0.0.10:6443"

[sites.alps.k8s.authentication.vault]
base_url = "https://vault.example.com:8200"
```

If you had a v1 `config.toml` with all the backend fields on the same
host, the migration mapping is:

```
copy these fields verbatim:        log, auditor, sites
add new [server] section:          listen_address, port, cert, key,
                                   console_inactivity_timeout_secs
drop (CLI-only):                   site, parent_hsm_group, hsm_group,
                                   manta_server_url
drop (no longer recognised):       audit_file (audit emission is
                                   Kafka-only via [auditor.kafka])
drop (no longer recognised):       sites.<X>.manta_server_url
```

### 2.3 TLS

`server.toml` `[server].cert` and `key` point at PEM files.
Self-signed is OK behind a reverse proxy that terminates TLS;
production should use a real certificate or a wildcard from your
site's CA. Without `cert`/`key`, the server runs plain HTTP — fine
for `localhost` smoke tests, never for a deployment.

### 2.4 Auth rate limiting

`[server].auth_rate_limit_per_minute` enforces a per-source-IP token
bucket on `/api/v1/auth/*`. Default is 60 req/min/IP; omit to
disable in-process limiting and rely on your reverse proxy. The
limiter is defence-in-depth — terminate at the proxy as well.

### 2.5 Logging

`log = "info"` covers most operations. `log = "debug"` adds the
backend-dispatch boundary, request-extractor decisions, and HTTP
round-trip timing. Useful for diagnosis; verbose otherwise.

Per-module overrides work via the standard `tracing-subscriber`
syntax: `log = "manta_server=debug,hyper=warn,info"`.

The server prints its full effective configuration to stdout on
startup so operators see exactly what got loaded — config file path,
listen address, TLS state, auth rate limit, audit file, per-site
backend URLs, k8s/vault URLs (no secrets ever logged).

### 2.6 Audit

`[auditor].kafka` if present streams every `/auth/*` outcome to a
Kafka topic. Same field shape as v1's audit block. If you don't run
Kafka, omit the section — auditing is silent.

### 2.7 Verify

```bash
# Health check (no auth)
curl -k https://manta-server.example.com:8443/health
# OpenAPI spec
curl -k https://manta-server.example.com:8443/openapi.json | jq .info
# Interactive docs
xdg-open https://manta-server.example.com:8443/docs
```

Then point a `cli.toml` at the server and run `manta config show`
from a workstation.

---

## 3. Integrators (shell scripts and programmatic HTTP clients)

### 3.1 Shell scripts calling `manta`

For scripts written against v1 syntax:

1. **Nothing breaks immediately.** Every renamed CLI shape works
   in v2 — the binary just prints a one-line stderr warning per
   invocation. Pipe `2>/dev/null` if your CI logs are noisy.
2. **Convert at your own pace.** Use the command/flag rename table
   in §1.4 above. The conversions are mechanical.
3. **Adopt `--output json` for parsing.** Wherever v1 scripts did
   `grep` or `awk` over textual output, v2 lets you `jq`:

   ```bash
   # v1
   manta add group --label compute > /tmp/out
   grep -q "created" /tmp/out || exit 1

   # v2
   manta add group --label compute --output json | jq -e '.status == "ok"'
   ```

4. **Drop direct backend URLs.** v1 scripts that hit the CSM API
   directly (e.g. `curl $SHASTA_BASE_URL/cfs/v3/sessions`) should now
   go through `manta-server`'s
   `https://<server>:8443/api/v1/sessions` endpoint — auth and
   token-handling are the server's job.

### 3.2 Programmatic HTTP clients

If you have a Python/Go/whatever client written against v1's direct
backend calls, switching to v2 means rewriting against
`manta-server`'s API. The full reference is in [API.md](API.md);
the headline:

- Base URL: `https://<host>:8443/api/v1`
- Required headers: `X-Manta-Site: <site>` + `Authorization: Bearer <token>`
- Auth bootstrap: `POST /api/v1/auth/token` with
  `{"username":"...","password":"..."}` → returns `{"token":"..."}`
- Error envelope: `{"error":"..."}` with conventional status codes
- OpenAPI spec served at `/openapi.json`; Swagger UI at `/docs`

REST endpoints that have changed names in the latest v2 release:

| Old path | New path |
|---|---|
| `GET /clusters` | `GET /groups/nodes` |
| `GET /hardware-clusters` | `GET /groups/hardware` |

Old paths still work for one release with a server-side warn log on
every hit; same query parameters, same response shape. The
[write-side `/hardware-clusters/{target}/...`](API.md#hardware-component-group-management)
paths are unchanged in this release.

---

## 4. Step-by-step playbook

For a typical site with N workstation users + one server host:

### Operator side (do first, once)

1. Provision a host for `manta-server`. Open TCP port 8443 (or
   whatever you choose) and configure DNS.
2. Install `manta-server` on the host (binary, distro package, or
   container). Confirm `manta-server --version`.
3. Author `~/.config/manta/server.toml` per [§2.2](#22-author-servertoml).
   Set TLS cert/key paths, backend URLs, k8s/vault URLs, audit
   destination.
4. Start the service (systemd, docker, …). Tail the logs and confirm
   you see the `[server] effective configuration` and one
   `[site] configured` line per site.
5. `curl -k https://<host>:8443/health` from another host to confirm
   reachability.
6. Hand out the server URL to your users.

### User side (each workstation, parallel)

1. Install the v2 `manta` binary. Confirm `manta --version`.
2. Move (or copy) `~/.config/manta/config.toml` to
   `~/.config/manta/cli.toml` and edit per [§1.2](#12-convert-your-config-file).
   Add `manta_server_url` pointing at the operator's URL.
3. `rm -rf ~/.cache/manta/` to flush v1 tokens.
4. Run `manta config show` — should print the loaded settings and
   the groups your token can access.
5. Run any command you used regularly in v1; if it warns about a
   deprecated form, jot down the new spelling. Optional: regenerate
   completions per [§1.6](#16-regenerate-your-shell-completion).

### Integrator side (continuous)

1. Update every call site that uses one of the removed v1 forms
   listed in §1.4 — those subcommands now error with `unrecognized
   subcommand`. Flag aliases (e.g. `--hsm-group`, `--target-cluster`)
   still work.
2. Write new scripts directly against the v2 shapes.

---

## Reference

- [README.md](README.md) — installation, deployment overview, operator runbook
- [CLI.md](CLI.md) — every command + every flag, with the migrating
  table at the top
- [API.md](API.md) — every REST endpoint, OpenAPI spec, troubleshooting curl recipes
- [GUIDE.md](GUIDE.md) — task-oriented walkthroughs (all examples use v2 syntax)
- [CHANGELOG.md](CHANGELOG.md) — full release-by-release diff
