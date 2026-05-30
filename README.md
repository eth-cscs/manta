[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/eth-cscs/manta)

# MANTA

Another CLI tool for [Alps](https://www.cscs.ch/science/computer-science-hpc/2021/cscs-hewlett-packard-enterprise-and-nvidia-announce-worlds-most-powerful-ai-capable-supercomputer).

## TL;DR

A command-line + HTTP API frontend for HPC clusters running [CSM](https://github.com/Cray-HPE/cray-site-init) or [OpenCHAMI](https://www.openchami.org/). Two independent binaries from one Cargo workspace:

- **`manta`** — interactive CLI. Forwards every operation (including auth) to a `manta-server` over HTTPS; never calls the backend directly.
- **`manta-server`** — Axum HTTPS server. Holds the per-site backend credentials and exposes a Swagger-documented REST + WebSocket API at `https://<host>:8443/api/v1` (default port).

**Get something running locally:**

```bash
# 1. clone and enter the repo
git clone https://github.com/eth-cscs/manta && cd manta

# 2. build both binaries
cargo build -p manta-cli -p manta-server

# 3. copy the example configs into manta's config directory
#    Linux: ~/.config/manta/
#    macOS: ~/Library/Application Support/local.cscs.manta/
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/manta"             # Linux
# CONFIG_DIR="$HOME/Library/Application Support/local.cscs.manta" # macOS
mkdir -p "$CONFIG_DIR"
cp cli.toml.example    "$CONFIG_DIR/cli.toml"      # edit
cp server.toml.example "$CONFIG_DIR/server.toml"   # edit

# 4. start the server, then drive it with the CLI
./target/debug/manta-server &
./target/debug/manta get sessions
```

| Where to look next | For |
|---|---|
| [GUIDE.md](GUIDE.md) | common workflows ("how do I deploy a SAT file?") |
| [CLI.md](CLI.md) | per-flag reference for every `manta` subcommand |
| [API.md](API.md) | REST + WebSocket endpoints, schemas, status codes |
| [MIGRATING.md](MIGRATING.md) | upgrading from manta v1 to v2 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | crate layout, module boundaries, security model |
| [docs.rs/manta-shared](https://docs.rs/manta-shared) | rustdoc for the shared library crate |
| [eth-cscs.github.io/manta](https://eth-cscs.github.io/manta/) | rustdoc for the whole workspace (rebuilt on every push to `main`) |

## Repository layout

manta is a Cargo workspace with three crates:

```
crates/
├── manta-shared/   (lib)  — wire types, common helpers, backend dispatcher
├── manta-cli/      (bin)  — terminal client (binary: `manta`)
└── manta-server/   (bin)  — Axum HTTPS server (binary: `manta-server`)
```

Build a single crate with `cargo build -p manta-cli` or `cargo build -p manta-server`; the two binaries do not depend on each other. See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Documentation

| Document | Description |
|----------|-------------|
| [GUIDE.md](GUIDE.md) | User guide — common workflows and practical examples |
| [CLI.md](CLI.md) | Full CLI command reference — every command, subcommand, and flag |
| [API.md](API.md) | HTTP API reference — REST and WebSocket endpoints |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Codebase architecture — for contributors |

Manta is a frontend cli to interact with CSM and OCHAMI.

## Deployment

### Prerequisites

Install build dependencies

```shell
$ cargo install cargo-release dist git-cliff
```

> `dist` is the renamed successor of `cargo-dist`; the old name still installs the same binary but emits a deprecation warning.

### Clone repo

```bash
git clone https://github.com/eth-cscs/manta && cd manta
```

The `main` branch holds the current 2.x line.

### Build container images

The two binaries ship as two images, each with its own multi-stage Dockerfile alongside its source. **Build from the workspace root in both cases** so the Cargo lockfile and shared sources are in the build context:

```
docker build -f crates/manta-cli/Dockerfile    -t manta-cli    .
docker build -f crates/manta-server/Dockerfile -t manta-server .
```

A `.dockerignore` at the workspace root keeps `target/`, `.git/`, and editor state out of the context.

#### Copy configuration file

The CLI reads `cli.toml`; the HTTP server reads `server.toml`. Both live in manta's config directory (`~/.config/manta/` on Linux, `~/Library/Application Support/local.cscs.manta/` on macOS). Each has its own schema — see the [Configuration files](#configuration-files) section below for the full layout. A minimal CLI config looks like:

```bash
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/manta"             # Linux
# CONFIG_DIR="$HOME/Library/Application Support/local.cscs.manta" # macOS
mkdir -p "$CONFIG_DIR"
cat > "$CONFIG_DIR/cli.toml" <<EOF
log = "info"

site = "ochami"
parent_hsm_group = "nodes_free"
audit_file = "/tmp/manta_audit.log"
manta_server_url = "https://manta-server.example.com:8443"   # required
EOF
```

The CLI config has no `[sites.*]` block — per-site backend connection details (URLs, TLS certs, k8s, vault) live in `server.toml`. The CLI's `site = "..."` value is just the `X-Manta-Site` header it sends on each request; the server validates it.

#### Start the `ochami` services from the [deployment recipe quickstart](https://github.com/OpenCHAMI/deployment-recipes/tree/main/quickstart).

> [!NOTE]
> Make sure to set the `ACCESS_TOKEN` environment variable and create a CA certificate in the same directory as the config file. This can be done using the convienience functions from the the OpenCHAMI deployment recipe repository.
>
> To set the `ACCESS_TOKEN` environment variable and create/renew the CA certificate (assuming you have cloned the deployment recipe quickstart):
> ```bash
> # collection of useful functions
> ochami_deployment_recipe_quickstart=path/to/quickstart
> source $ochami_deployment_recipe_quickstart/bash_functions.sh
>
> # set environment variable then create the cert
> export ACCESS_TOKEN=$(gen_access_token)
> get_ca_cert > "$CONFIG_DIR/ochami_root_cert.pem"   # $CONFIG_DIR set above
> ```

#### Run the CLI with one of the two options mentioned above to confirm that `manta` is working.

The `manta-cli` image has `manta` as its ENTRYPOINT, so anything after the image tag is forwarded as CLI args:

```bash
docker run -it --network=host \
  -v "$CONFIG_DIR":/root/.config/manta \
  -e MANTA_CSM_TOKEN \
  manta-cli get redfish-endpoints
```

The `manta-server` image runs the HTTPS server; mount your config + TLS material and publish the port:

```bash
docker run -p 8443:8443 \
  -v "$CONFIG_DIR":/root/.config/manta:ro \
  -v /etc/manta/tls:/etc/manta/tls:ro \
  manta-server
```

> [!NOTE]
> Some commands will not work yet with OpenCHAMI services and will sometimes show a message indicating no implementation for the backend.
>
> ```bash
> docker run -it --rm --network=host \
>   -v "$CONFIG_DIR":/root/.config/manta \
>   -e MANTA_CSM_TOKEN \
>   manta-cli get sessions
> ERROR | Get and filter sessions command not implemented for this backend
> exit status 1
> ```
>
> Some other commands may fail simply because CSM-only services are not part of an OpenCHAMI deployment.

### HTTP server mode

Manta can run as an HTTPS server, exposing all CLI operations as a REST + WebSocket API. This is useful for automation, scripting, and integration with other tools without requiring direct CLI access.

The HTTP server lives in its own binary (`manta-server`) inside the `crates/manta-server` workspace member. Build it with `cargo build -p manta-server`.

**Start the server**

Write `server.toml` in manta's config directory first — see the [Configuration files](#configuration-files) section below for the path on your platform. Then:

```bash
manta-server
```

Each setting in the `[server]` block can be overridden at runtime:

| Flag | Overrides | Description |
|------|-----------|-------------|
| `--port` | `[server].port` | Port to listen on |
| `--listen-address` | `[server].listen_address` | Bind address |
| `--cert` | `[server].cert` | TLS certificate path |
| `--key` | `[server].key` | TLS private key path |

> The CLI no longer ships a `manta serve` subcommand — invoke `manta-server` directly.

---

### Configuration files

Manta reads two TOML files, one per binary: `cli.toml` for the CLI and `server.toml` for the HTTP server. Both live in manta's config directory:

- Linux: `~/.config/manta/` (or `$XDG_CONFIG_HOME/manta/` if set)
- macOS: `~/Library/Application Support/local.cscs.manta/`

Override the path with `MANTA_CLI_CONFIG` / `MANTA_SERVER_CONFIG`.

The two schemas are **disjoint**: the CLI's `cli.toml` carries only the CLI-side knobs (`site`, `parent_hsm_group`, `manta_server_url`, optional `socks5_proxy`, optional `[auditor.kafka]`) — it has **no `[sites.*]` block**. Every per-site backend connection detail (URLs, TLS certs, k8s, vault, per-site SOCKS proxies) lives in `server.toml`, alongside the `[server]` block (TLS, listen address, console timeout, auth rate limit).

**`cli.toml`**

`manta_server_url` is required: the CLI no longer talks to CSM/OCHAMI backends directly — every operation (including auth) is forwarded to the named manta server. Run `manta-server` on a reachable host first.

```toml
log = "info"
audit_file = "/var/log/manta/cli-audit.log"

site             = "alps"                                # active site (X-Manta-Site header)
parent_hsm_group = "nodes_free"
manta_server_url     = "https://manta-server.cscs.ch:8443"   # required
socks5_proxy         = "socks5h://127.0.0.1:1080"            # optional: reaches manta-server
request_timeout_secs = 600                                   # optional: caps long-running CLI HTTP calls (e.g. `manta power`); omit for no client-side timeout

[auditor.kafka]
brokers = ["kafka.cscs.ch:9095"]
topic   = "manta-cli-audit"
```

The CLI has no `[sites]` section: it only knows about the one
`manta-server` it talks to. Per-site backend connection details
(URLs, TLS certs, k8s, vault, per-site SOCKS proxies) live entirely
in `server.toml`.

**`server.toml`**

```toml
log = "info"
audit_file = "/var/log/manta/server-audit.log"

[server]
listen_address                  = "0.0.0.0"   # optional; default 0.0.0.0
port                            = 8443        # optional; default 8443 if cert+key set, else 8080
cert                            = "/etc/manta/tls/server.crt"
key                             = "/etc/manta/tls/server.key"
console_inactivity_timeout_secs = 1800
auth_rate_limit_per_minute      = 60      # per source IP for /api/v1/auth/*; omit to disable
request_timeout_secs            = 60      # global per-route timeout (returns 408); default 60
power_timeout_secs              = 600     # per-route override for POST /power (cluster reset can run for minutes); default 600

[auditor.kafka]
brokers = ["kafka.cscs.ch:9095"]
topic   = "manta-server-audit"

[sites.alps]
backend           = "csm"
shasta_base_url   = "https://api.alps.cscs.ch"
root_ca_cert_file = "/etc/manta/certs/alps_root_cert.pem"
socks5_proxy      = "socks5h://127.0.0.1:1080"   # optional: per-site backend proxy

[sites.alps.k8s]
api_url = "https://10.0.0.10:6443"

[sites.alps.k8s.authentication.vault]
base_url = "https://vault.cscs.ch:8200"          # also used by sat-file/session handlers
```

The runtime Vault URL is derived from `[sites.X.k8s.authentication.vault].base_url` at startup; the vault secret path is computed from a hard-coded prefix and the site name. No standalone `vault_base_url` / `vault_secret_path` keys.

See `cli.toml.example` and `server.toml.example` at the workspace root for fully-commented templates.

**Migrating from the pre-split `config.toml`**

There is no auto-migration command. When either binary starts and finds its config file missing, it prints a minimal example and — if a legacy `config.toml` exists in manta's config directory — a field-by-field mapping of what to copy where. Copy by hand following that mapping. The per-site `sites.X.manta_server_url` field was removed; use the top-level `manta_server_url` in `cli.toml` if you need it.

**Example — list CFS sessions**

```bash
curl -sk -H "Authorization: Bearer $TOKEN" \
  https://localhost:8443/api/v1/sessions | jq .
```

**Example — open a node console (WebSocket)**

```bash
wscat -H "Authorization: Bearer $TOKEN" \
  --connect wss://localhost:8443/api/v1/nodes/x3000c0s1b0n0/console
```

See [API.md](API.md) for the full endpoint reference, or browse the interactive Swagger UI at `https://localhost:8443/docs` once the server is running.

---

### Build from sources

Install Rust toolchain https://www.rust-lang.org/tools/install

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install cross to be able to complile on different platforms

```shell
cargo install cross
```

Generate binary (cross compilation)

```shell
scripts/build
```

or

```shell
rustup target add x86_64-unknown-linux-gnu
cargo build --target=x86_64-unknown-linux-gnu
```

### Development

#### Prerequisites

Install `dist` and `cargo-release`:

```
cargo install dist
cargo install cargo-release
```

Configure `dist`. Accept default options and only target linux assets:

```
dist init -t $(uname -m)-unknown-$(uname -s | tr '[:upper:]' '[:lower:]')-gnu
```

Then remove the assets for macos and windows

Make sure a github workflow is created in `.github/workflows/release.yml`

#### Deployment

This project is already integrated with github actions through 'cargo release' and 'git cliff'

> git cliff will parse your commits and update the CHANGELOG.md file automatically as long as your commits follow [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#specification). The commit-type → section mapping lives in the `[git.commit_parsers]` table inside [`cliff.toml`](cliff.toml).

```
cargo release <bump level> --execute
```

> choose your [bump level](https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md#bump-level) accordingly

If everything went well, the binaries will be at `target/x86_64-unknown-linux-gnu/release/manta-cli` and `target/x86_64-unknown-linux-gnu/release/manta-server`.

### Profiling

#### Enable capabilities

```bash
sudo sysctl -w kernel.perf_event_paranoid=-1
```

#### Install perf

```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

#### Grant access to kernel address map

```bash
sudo sh -c " echo 0 > /proc/sys/kernel/kptr_restrict"
```

#### Create perf data

```bash
perf stat -ad -r 100 target/release/manta-cli get sessions
```

#### Identify bottlenecks and get hotspots for those events


```bash
perf record -g --call-graph=dwarf -F max target/release/manta-cli get sessions
```

#### Convert perf data file to a format firefox profiles understands

```bash
perf script -F +pid > manta.perf
```

Go to https://profiler.firefox.com/ and open manta.perf file


<!--
  DHAT memory profiling used to live here. It required a `dhat-heap`
  feature in manta-cli's Cargo.toml that no longer exists. If you want
  to revive it, add `dhat = "..."` as an optional dep, expose a
  `dhat-heap` feature that enables it, and wrap `main` in a
  `#[cfg(feature = "dhat-heap")] let _profiler = dhat::Profiler::new_heap()`.
  Then `cargo run -r --features dhat-heap -- get sessions` will produce
  dhat-heap.json viewable at https://nnethercote.github.io/dh_view/dh_view.html
-->

