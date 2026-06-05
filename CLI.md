# Manta CLI Reference

Complete reference for all `manta` commands, subcommands, and flags.

**Binary name:** the CLI binary is `manta` (the Cargo package is `manta-cli`; the `[[bin]]` block in its manifest renames the produced executable). The HTTP API server is a separate binary (`manta-server`); see [API.md](API.md) for its endpoints and [README.md](README.md) for how to run it.

## TL;DR

Every command takes one of the top-level verbs:

| Verb | Purpose |
|---|---|
| [`config`](#config) | Manage `~/.config/manta/cli.toml` |
| [`get`](#get) | Read-only queries (sessions, configurations, nodes, groups, hardware, …) |
| [`add`](#add) | Create resources (node, group, hardware components, boot/kernel params) |
| [`apply`](#apply) | Apply changes (SAT files, boot/kernel parameters, Redfish endpoints, hardware allocation, …) |
| [`run`](#run) | Create and run jobs (configuration sessions) |
| [`delete`](#delete) | Remove resources |
| [`backup`](#backup) | Back up cluster state (virtual cluster) |
| [`restore`](#restore) | Restore cluster state from a backup |
| [`migrate`](#migrate) | Move nodes between groups |
| [`power`](#power) | Power on/off/reset nodes or groups |
| [`log`](#log-value) | Stream CFS session logs (alias: `manta logs`) |
| [`console`](#console) | Open a WebSocket console to a node or running CFS session |
| [`gen-autocomplete`](#gen-autocomplete) | Generate shell completion scripts |
| [`gen-man`](#gen-man) | Generate and install man pages |
| [`upgrade`](#upgrade) | Replace this `manta` binary with the latest release |

**Global flag** (available on every command):

| Flag | Description |
|------|-------------|
| `--site <SITE_NAME>` | Override the active site from config for this invocation |

## Migrating from earlier shapes

Several CLI shapes were renamed in the latest releases and the old
forms have now been removed. The canonical names:

| Removed form | Replacement |
|---|---|
| `manta apply session` | `manta run session` |
| `manta apply boot cluster` | `manta apply boot group` |
| `manta apply hardware cluster` | `manta apply hardware group` |
| `manta get cluster` | `manta get group-nodes` |
| `manta get hardware cluster` | `manta get group-hardware` |
| `manta power on cluster` | `manta power on group` |
| `manta power off cluster` | `manta power off group` |
| `manta power reset cluster` | `manta power reset group` |
| `manta migrate vCluster backup` | `manta backup vcluster` |
| `manta migrate vCluster restore` | `manta restore vcluster` |
| `manta config gen-autocomplete` | `manta gen-autocomplete` |
| `manta update boot-parameters` | `manta apply boot-parameters` |
| `manta update redfish-endpoints` | `manta apply redfish-endpoint` |
| `manta add-nodes-to-groups` | `manta add nodes` |
| `manta remove-nodes-from-groups` | `manta delete nodes` |

The following flag renames are still in effect; the old spellings
are kept as visible clap aliases on the canonical commands:

| Old flag | New flag |
|---|---|
| `--target-cluster` | `--target-group` |
| `--parent-cluster` | `--parent-group` |
| `--create-hsm-group` | `--create-group` |
| `--delete-hsm-group` | `--delete-group` |
| `--create-target-hsm-group` | `--create-target-group` |
| `--delete-empty-parent-hsm-group` | `--delete-empty-parent-group` |
| `--hsm-group` | `--group` |
| `redfish-endpoint` (singular noun on `add` / `delete` / `apply`) | `redfish-endpoints` (plural) |

---

## config

Manage the local manta configuration file (`~/.config/manta/cli.toml`).

### config show

Print current configuration values.

```
manta config show
```

### config set hsm \<HSM_GROUP_NAME\>

Set the default target HSM group.

```
manta config set hsm compute
```

### config set parent-hsm \<HSM_GROUP_NAME\>

Set the parent (resource pool) HSM group.

```
manta config set parent-hsm nodes_free
```

### config set site \<SITE_NAME\>

Set the active site (must match a `[sites.<name>]` entry in config).

```
manta config set site cscs_prod
```

### config set log \<LOG_LEVEL\>

Set log verbosity. Values: `error`, `warn`, `info`, `debug`, `trace`.

```
manta config set log debug
```

### config unset hsm

Remove the default HSM group.

### config unset parent-hsm

Remove the parent HSM group.

### config unset auth

Remove the stored authentication token.

---

## get

Retrieve information from the cluster.

### get groups \[VALUE\]

List HSM groups, or show details for a single group.

| Arg/Flag | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `VALUE` | string | no | — | Group name; returns all groups if omitted |
| `-o/--output` | string | no | `table` | Output format: `table`, `json` |

```
manta get groups
manta get groups my-cluster -o json
```

### get group-hardware \<GROUP_NAME\>

List hardware components in a group.

| Arg/Flag | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `GROUP_NAME` | string | **yes** | — | HSM group name |
| `-o/--output` | string | no | `summary` | Output format: `summary`, `details`, `pattern`, `json` |

Output modes:
- `summary` — aggregated table: hw component → total count across all nodes
- `details` — per-node table with one column per unique hw component type
- `pattern` — single line `<group>:<component>:<qty>:...`
- `json` — raw JSON

```
manta get group-hardware gpu-cluster -o details
```

### get hardware nodes \<VALUE\>

Per-node hardware component breakdown for an explicit list of nodes. Equivalent to `get group-hardware --output details` but scoped to specific nodes instead of an entire group.

| Arg/Flag | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `VALUE` | string | **yes** | — | Xnames/nids or [hostlist expression](#node-expressions) |
| `-o/--output` | string | no | `table` | Output format: `table`, `json` |

```
manta get hardware nodes 'x3000c0s1b0n[0-3]'
manta get hardware nodes x3000c0s1b0n0,x3000c0s1b0n1 -o json
```

### get sessions

List CFS sessions with optional filters.

| Flag | Type | Description |
|------|------|-------------|
| `-n/--name` | string | Exact session name |
| `-a/--min-age` | string | Minimum age, e.g. `1d`, `6h` |
| `-A/--max-age` | string | Maximum age |
| `-t/--type` | string | `image` or `runtime` |
| `-s/--status` | string | `pending`, `running`, `complete` |
| `-m/--most-recent` | flag | Return only the most recent session |
| `-l/--limit` | u8 | Return the last N sessions |
| `-x/--xnames` | string | Comma-separated xnames to filter by |
| `-H/--group` | string | HSM group name to filter by |
| `-o/--output` | string | Output format: `json` |

> `--group`, `--xnames`, and `--name` are mutually exclusive.  
> `--most-recent` and `--limit` are mutually exclusive.

```
manta get sessions --group compute --status running
manta get sessions --most-recent -o json
```

### get configurations

List CFS configurations.

| Flag | Type | Description |
|------|------|-------------|
| `-n/--name` | string | Exact configuration name |
| `-p/--pattern` | string | Glob pattern for configuration name |
| `-m/--most-recent` | flag | Show only the most recent |
| `-l/--limit` | u8 | Return the last N configurations |
| `-H/--group` | string | HSM group to filter by |
| `-o/--output` | string | Output format: `json` |

> `--group` and `--name` are mutually exclusive.  
> `--most-recent` and `--limit` are mutually exclusive.

```
manta get configurations --pattern "csm-config-*" --limit 5
```

### get templates

List BOS session templates.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-n/--name` | string | — | Template name |
| `-m/--most-recent` | flag | — | Show most recent only |
| `-l/--limit` | u8 | — | Return last N templates |
| `-H/--group` | string | — | HSM group name |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### get group-nodes \<GROUP_NAME\>

Show node membership and status for an HSM group.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `GROUP_NAME` | string | — | HSM group name |
| `-n/--nids-only-one-line` | flag | — | Print NIDs as a single comma-separated line |
| `-x/--xnames-only-one-line` | flag | — | Print xnames as a single comma-separated line |
| `-s/--status` | string | — | Filter by status: `OFF`, `ON`, `READY`, `STANDBY`, `PENDING`, `FAILED`, `CONFIGURED` |
| `-T/--summary-status` | flag | — | Print overall status summary |
| `-o/--output` | string | `table` | Output format: `table`, `table-wide`, `json`, `summary` |

```
manta get group-nodes compute -o summary
manta get group-nodes compute --xnames-only-one-line
```

### get nodes \<VALUE\>

Show details for specific nodes.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `VALUE` | string | — | Comma-separated xnames/nids or [hostlist expression](#node-expressions) |
| `-n/--nids-only-one-line` | flag | — | Print NIDs as a single line |
| `-s/--status` | string | — | Filter by status |
| `-T/--summary-status` | flag | — | Print status summary |
| `-S/--include-siblings` | flag | — | Include nodes sharing the same power supply |
| `-o/--output` | string | `table` | Output format: `table`, `table-wide`, `json`, `summary` |

```
manta get nodes 'x3000c0s1b0n[0-3]'
manta get nodes nid001313,nid001314 -o json
```

### get images

List IMS images, sorted by creation time.

| Flag | Type | Description |
|------|------|-------------|
| `-i/--id` | string | Specific image ID |
| `-p/--pattern` | string | Regex matched against image name |
| `-m/--most-recent` | flag | Show most recent only |
| `-l/--limit` | u8 | Return last N images |

### get boot-parameters

Get BSS boot parameters.

| Flag | Type | Description |
|------|------|-------------|
| `-H/--group` | string | HSM group name |
| `-n/--nodes` | string | Comma-separated xnames/nids or hostlist expression |

> Pass one of `--group` or `--nodes` — clap doesn't enforce this for you, so omitting both returns the global set.

```
manta get boot-parameters --group compute
```

### get kernel-parameters

Get kernel boot parameters for nodes.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-n/--nodes` | string | — | Comma-separated xnames/nids or hostlist |
| `-H/--group` | string | — | HSM group name |
| `-f/--filter` | string | — | Comma-separated parameter names to extract, e.g. `console,crashkernel` |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

> One of `--nodes` or `--group` is required.

```
manta get kernel-parameters --group compute --filter console,loglevel
```

### get redfish-endpoints

List registered Redfish endpoints (BMCs).

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-i/--id` | string | — | Filter by xname |
| `-f/--fqdn` | string | — | Filter by FQDN |
| `-u/--uuid` | string | — | Filter by UUID |
| `-m/--macaddr` | string | — | Filter by MAC address |
| `-I/--ipaddress` | string | — | Filter by IP address |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

---

## add

Create new resources.

### add group

Create a new HSM group.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-l/--label` | string | **yes** | Group name |
| `-d/--description` | string | no | Group description |
| `-n/--nodes` | string | no | Initial members (xnames/nids/hostlist) |

```
manta add group --label my-cluster --description "GPU nodes" --nodes 'x3000c0s1b0n[0-7]'
```

### add node

Register a brand-new node with HSM. Distinct from [`add nodes`](#add-nodes) (plural), which assigns existing nodes to an HSM group.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--id` | string | **yes** | Node xname |
| `-g/--group` | string | **yes** | Group the node belongs to |
| `-H/--hardware` | path | no | File with hardware info |
| `-a/--arch` | string | no | Architecture: `X86`, `ARM`, `Other` |
| `-d/--disabled` | flag | no | Disable node on creation |

### add nodes

Add existing nodes to an HSM group's membership. Distinct from `add node` (singular), which registers a brand-new node in the inventory.

> Replaces the removed `manta add-nodes-to-groups` top-level command.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-g/--group` | string | **yes** | Group to add the nodes to |
| `-n/--nodes` | string | **yes** | Nodes to add (xnames/nids/hostlist) |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-o/--output` | string | no | Output format: `table`, `json` (default `table`) |

### add hardware *(WIP)*

Add hardware components to an HSM group.

> Flag names below are the canonical group-centric form; the old
> `--target-cluster`, `--parent-cluster`, `--create-hsm-group`
> spellings continue to work as visible aliases.

| Flag | Type | Description |
|------|------|-------------|
| `-P/--pattern` | string | Hardware pattern, e.g. `a100:4:epyc:2` |
| `-t/--target-group` | string | Target group name |
| `-p/--parent-group` | string | Source group providing the hardware |
| `-d/--dry-run` | flag | Simulate without changes |
| `-c/--create-group` | flag | Create target group if it does not exist |

### add boot-parameters

Add BSS boot parameters for nodes.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-H/--hosts` | string | **yes** | Comma-separated xnames |
| `-n/--nids` | string | no | Comma-separated NIDs |
| `-m/--macs` | string | no | Comma-separated MAC addresses |
| `-p/--params` | string | no | Kernel parameters string |
| `-k/--kernel` | string | no | S3 path to kernel file |
| `-i/--initrd` | string | no | S3 path to initrd file |
| `-c/--cloud-init` | string | no | Cloud-init script |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-y/--assume-yes` | flag | no | Non-interactive mode |

### add kernel-parameters \<VALUE\>

Merge kernel parameters into existing values for nodes.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Space-separated `key=value` pairs |
| `-n/--nodes` | string | no* | Target nodes (xnames/nids/hostlist) |
| `-H/--group` | string | no* | Target HSM group |
| `-O/--overwrite` | flag | no | Overwrite existing values |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip node reboot after change |
| `-d/--dry-run` | flag | no | Simulate without changes |

> One of `--nodes` or `--group` is required.

```
manta add kernel-parameters "console=ttyS0,115200 loglevel=7" --group compute
```

### add redfish-endpoint

Register a Redfish endpoint (BMC).

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--id` | string | **yes** | Xname (physical location ID) |
| `-n/--name` | string | no | Human-readable name |
| `-H/--hostname` | string | no | Hostname portion of FQDN |
| `-d/--domain` | string | no | Domain portion of FQDN |
| `-f/--fqdn` | string | no | Full FQDN |
| `-e/--enabled` | flag | no | Enable the endpoint |
| `-u/--user` | string | no | BMC username |
| `-p/--password` | string | no | BMC password |
| `-U/--use-ssdp` | flag | no | Use SSDP for discovery |
| `-m/--mac-required` | flag | no | Require MAC for geolocation |
| `-M/--macaddr` | string | no | MAC address (colon-separated) |
| `-I/--ipaddress` | string | no | IP address (IPv4 or IPv6) |
| `-r/--rediscover-on-update` | flag | no | Trigger rediscovery on update |
| `-t/--template-id` | string | no | Discovery template ID |

---

## apply

Apply changes to the system.

### apply sat-file

Process a SAT (System Admin Toolkit) file. This is the primary workflow for deploying configurations, building images, and applying session templates.

| Flag | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `-t/--sat-template-file` | path | **yes** | — | SAT file path (may be a Jinja2 template) |
| `-f/--values-file` | path | no | — | Jinja2 values file |
| `-V/--values` | string… | no | — | Inline Jinja2 values; overrides `--values-file` |
| `--reboot` | flag | no | — | Reboot nodes after session templates are applied |
| `-v/--ansible-verbosity` | 1–4 | no | `2` | Ansible verbosity level |
| `-P/--ansible-passthrough` | string | no | — | Extra Ansible parameters (`--extra-vars`, `--forks`, etc.) |
| `-o/--overwrite-configuration` | flag | no | — | Overwrite existing CFS configurations |
| `-w/--watch-logs` | flag | no | — | Stream Ansible logs to stdout |
| `-T/--timestamps` | flag | no | — | Include timestamps in log output |
| `-i/--image-only` | flag | no | — | Process only `configurations` + `images` sections |
| `-s/--sessiontemplate-only` | flag | no | — | Process only `configurations` + `session_templates` sections |
| `-p/--pre-hook` | string | no | — | Shell command to run before processing |
| `-a/--post-hook` | string | no | — | Shell command to run on success |
| `-y/--assume-yes` | flag | no | — | Non-interactive mode |
| `-d/--dry-run` | flag | no | — | Simulate without making changes |

```
manta apply sat-file -t my-cluster.yaml -f values.yaml --watch-logs
manta apply sat-file -t deploy.yaml -i   # configurations + images only
manta apply sat-file -t deploy.yaml -s   # configurations + session templates only
```

### apply template

Create a BOS session from an existing BOS session template.

| Flag | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `-t/--template` | string | **yes** | — | BOS session template name |
| `-l/--limit` | string | **yes** | — | Comma-separated nodes, groups, or roles |
| `-n/--name` | string | no | — | Session name |
| `-o/--operation` | string | no | `reboot` | `reboot`, `boot`, or `shutdown` |
| `-i/--include-disabled` | flag | no | — | Include disabled nodes |
| `-y/--assume-yes` | flag | no | — | Non-interactive mode |
| `-d/--dry-run` | flag | no | — | Simulate without changes |

```
manta apply template -t my-template -l compute --operation reboot
```

### apply boot group \<GROUP_NAME\>

Update boot parameters (image + runtime config + kernel parameters) for all nodes in an HSM group.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `GROUP_NAME` | string | **yes** | HSM group name |
| `-i/--boot-image` | string | no* | Specific IMS image ID |
| `-b/--boot-image-configuration` | string | no* | CFS configuration name to derive boot image from |
| `-r/--runtime-configuration` | string | no | CFS configuration for post-boot configuration |
| `-k/--kernel-parameters` | string | no | Kernel parameters string |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Update boot params without rebooting |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-o/--output` | string | no | Output format: `table`, `json` (default `table`) |

> `--boot-image` and `--boot-image-configuration` are mutually exclusive.

```
manta apply boot group compute -b csm-config-2024 -r csm-config-2024
```

### apply boot nodes \<VALUE\>

Update boot parameters for specific nodes.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Comma-separated xnames/nids or hostlist expression |
| `-i/--boot-image` | string | no* | Specific IMS image ID |
| `-b/--boot-image-configuration` | string | no* | CFS configuration name |
| `-r/--runtime-configuration` | string | no | Post-boot CFS configuration |
| `-k/--kernel-parameters` | string | no | Kernel parameters |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip reboot |
| `-d/--dry-run` | flag | no | Simulate without changes |

### apply boot-parameters

Update existing BSS boot parameters for nodes — set the kernel
command line, kernel image, and/or initrd that nodes use on next
boot.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-H/--hosts` | string | **yes** | Comma-separated xnames |
| `-p/--params` | string | no | Kernel parameters |
| `-k/--kernel` | string | no | S3 path to kernel file |
| `-i/--initrd` | string | no | S3 path to initrd file |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-y/--assume-yes` | flag | no | Non-interactive mode |

### apply redfish-endpoint

Update an existing Redfish endpoint (network/auth/discovery fields).
Accepts the same flags as [`add redfish-endpoint`](#add-redfish-endpoint);
`-i/--id` is required to identify the entry.

### apply kernel-parameters \<VALUE\>

Replace kernel parameters for nodes (full replace, not merge).

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Space-separated `key=value` pairs |
| `-n/--nodes` | string | no* | Target nodes (xnames/nids/hostlist) |
| `-H/--group` | string | no* | Target HSM group |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip reboot |
| `-d/--dry-run` | flag | no | Simulate without changes |

> To merge parameters instead of replacing, use [`add kernel-parameters`](#add-kernel-parameters-value).

### apply ephemeral-environment

Launch a temporary container environment from an IMS image, useful for image inspection and debugging.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--image-id` | string | **yes** | IMS image ID to use as the container image |

### apply hardware group *(WIP)*

Upscale or downscale hardware resources in an HSM group.

> The old flag spellings (`--target-cluster`, `--parent-cluster`,
> `--create-target-hsm-group`, `--delete-empty-parent-hsm-group`)
> still work as visible aliases on the new flag names below.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-P/--pattern` | string | **yes** | Hardware pattern e.g. `a100:12:epyc:5` |
| `-t/--target-group` | string | **yes** | Group being reconfigured |
| `-p/--parent-group` | string | **yes** | Group offering or receiving freed resources |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-c/--create-target-group` | flag | no | Create target group if it does not exist |
| `-D/--delete-empty-parent-group` | flag | no | Delete parent group if it becomes empty |
| `-u/--unpin-nodes` | flag | no | Allow any available nodes instead of pinned ones |
| `-o/--output` | string | no | Output format: `table`, `json` (default `table`) |

---

## run

Create and run jobs against the backend.

### run session

Create and run a CFS session from one or more local git repositories.

| Flag | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `-n/--name` | string | **yes** | — | Session name |
| `-r/--repo-path` | path… | **yes** | — | Path(s) to local git repos (repeatable) |
| `-l/--ansible-limit` | string | **yes** | — | Target xnames |
| `-H/--group` | string | no | — | HSM group scope (must be a superset of `--ansible-limit` if both are given) |
| `-p/--playbook-name` | string | no | `site.yml` | Ansible playbook filename |
| `-w/--watch-logs` | flag | no | — | Stream session logs |
| `-t/--timestamps` | flag | no | — | Show timestamps in logs |
| `-v/--ansible-verbosity` | 0–4 | no | `2` | Ansible verbosity |
| `-P/--ansible-passthrough` | string | no | — | Extra Ansible parameters |
| `-o/--output` | string | no | `table` | Output format: `table`, `json` |

```
manta run session -n my-session -r ~/repos/csm-config -l x3000c0s1b0n0
```

---

## delete

Remove resources from the system.

### delete group \<VALUE\>

Delete an HSM group. The group must be empty first.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Group name |
| `-f/--force` | flag | no | Force deletion |

### delete node \<VALUE\>

Remove a node from HSM entirely. Distinct from [`delete nodes`](#delete-nodes) (plural), which removes nodes from a group's membership without touching the inventory.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Node xname |

### delete nodes

Remove nodes from an HSM group's membership. Distinct from `delete node` (singular), which removes the node from the system inventory entirely.

> Replaces the removed `manta remove-nodes-from-groups` top-level command.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-g/--group` | string | **yes** | Group to remove the nodes from |
| `-n/--nodes` | string | **yes** | Nodes to remove (xnames/nids/hostlist) |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-o/--output` | string | no | Output format: `table`, `json` (default `table`) |

### delete configurations

Delete CFS configurations and all derivatives (sessions, BOS templates, IMS images).

| Flag | Type | Description |
|------|------|-------------|
| `-n/--configuration-name` | string | Glob pattern for configuration name |
| `-s/--since` | date | Delete configs updated after this date (`YYYY-MM-DD`) |
| `-u/--until` | date | Delete configs updated before this date |
| `-y/--assume-yes` | flag | Non-interactive mode |

> `--since` and `--until` must be used together.  
> `--configuration-name` and date range are mutually exclusive.

```
manta delete configurations --configuration-name "old-config-*" --assume-yes
manta delete configurations --since 2024-01-01 --until 2024-06-01
```

### delete session \<SESSION_NAME\>

Delete a CFS session. For image sessions, the associated IMS image is also deleted.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `SESSION_NAME` | string | **yes** | Session name |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `-d/--dry-run` | flag | no | Simulate without changes |

### delete images \<IMAGE_LIST\> *(WIP)*

Delete IMS images.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `IMAGE_LIST` | string | **yes** | Comma-separated image IDs |
| `-d/--dry-run` | flag | no | Simulate without changes |

### delete kernel-parameters \<VALUE\>

Remove specific kernel parameters from nodes.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Comma-separated parameter names to remove |
| `-n/--nodes` | string | no* | Target nodes (xnames/nids/hostlist) |
| `-H/--group` | string | no* | Target HSM group |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip reboot |
| `-d/--dry-run` | flag | no | Simulate without changes |

> One of `--nodes` or `--group` is required.

### delete boot-parameters

Delete BSS boot parameters for nodes.

| Flag | Type | Description |
|------|------|-------------|
| `-H/--hosts` | string | Comma-separated xnames |

### delete hardware *(WIP)*

Remove hardware components from a group, returning them to the parent pool.

> Flag names below are the canonical group-centric form; the old
> `--target-cluster`, `--parent-cluster`, `--delete-hsm-group`
> spellings continue to work as visible aliases.

| Flag | Type | Description |
|------|------|-------------|
| `-P/--pattern` | string | Hardware pattern |
| `-t/--target-group` | string | Group losing resources |
| `-p/--parent-group` | string | Group receiving freed resources |
| `-d/--dry-run` | flag | Simulate without changes |
| `-D/--delete-group` | flag | Delete the target group if it becomes empty |

### delete redfish-endpoint

Delete a registered Redfish endpoint.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--id` | string | **yes** | Xname of the endpoint |

---

## migrate

Move compute nodes between HSM groups.

> Backup/restore for virtual clusters has moved to the top-level
> [`backup`](#backup) and [`restore`](#restore) verbs. The old
> `manta migrate vCluster backup` / `restore` forms still work for
> one release but print a deprecation warning on use.

### migrate nodes \<XNAMES\>

Move compute nodes from one HSM group to another.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `XNAMES` | string | **yes** | Xnames/nids or [hostlist expression](#node-expressions) |
| `-t/--to` | string | **yes** | Destination HSM group |
| `-f/--from` | string | no | Source HSM group |
| `-d/--dry-run` | flag | no | Simulate without changes |

```
manta migrate nodes 'x3000c0s1b0n[0-3]' --to gpu-cluster --from nodes_free
```

---

## backup

Back up cluster state. Replaces `manta migrate vCluster backup`.

### backup vcluster

Backup a virtual cluster's full configuration (BOS, CFS, IMS image, HSM group).

| Flag | Type | Description |
|------|------|-------------|
| `-b/--bos` | string | BOS session template name |
| `-d/--destination` | dir | Destination folder for backup files |
| `-p/--pre-hook` | string | Shell command to run before backup |
| `-a/--post-hook` | string | Shell command to run after backup |
| `-o/--output` | string | Output format: `table`, `json` (default `table`) |

---

## restore

Restore cluster state from a backup. Replaces `manta migrate vCluster restore`.

### restore vcluster

Restore a virtual cluster from backup files.

| Flag | Type | Description |
|------|------|-------------|
| `-b/--bos-file` | path | BOS session template backup file |
| `-c/--cfs-file` | path | CFS configuration backup file |
| `-j/--hsm-file` | path | HSM group description backup file |
| `-m/--ims-file` | path | IMS backup file |
| `-i/--image-dir` | dir | Directory containing image files |
| `-p/--pre-hook` | string | Shell command before restore |
| `-a/--post-hook` | string | Shell command after restore |
| `-o/--overwrite` | flag | Overwrite existing data |
| `--output` | string | Output format: `table`, `json` (default `table`) |

---

## power

Manage node power state.

Every `power on/off/reset` invocation first POSTs to the manta server, which kicks off a PCS transition and returns the transition id immediately. The CLI then polls the snapshot endpoint every 3 seconds (up to 15 minutes) and prints a one-line progress summary on each poll (`status: …, failed: …, in-progress: …, succeeded: …, total: …`). The command exits non-zero if any task in the transition failed. Pass `--no-wait` to skip the polling — the transition id is printed and the command exits 0.

### power on group \<GROUP_NAME\>

Power on all nodes in an HSM group.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `GROUP_NAME` | string | — | HSM group name |
| `-R/--reason` | string | — | Reason for the power action |
| `-y/--assume-yes` | flag | — | Non-interactive mode |
| `--no-wait` | flag | — | Return the transition id immediately; don't poll for completion |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### power on nodes \<VALUE\>

Power on specific nodes.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `VALUE` | string | — | Xnames/nids or hostlist expression |
| `-y/--assume-yes` | flag | — | Non-interactive mode |
| `--no-wait` | flag | — | Return the transition id immediately; don't poll for completion |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### power off group \<GROUP_NAME\> / power off nodes \<VALUE\>

Power off nodes.

| Arg/Flag | Type | Description |
|----------|------|-------------|
| `-g/--graceful` | flag | Graceful shutdown |
| `-R/--reason` | string | Reason for power-off (group only) |
| `-y/--assume-yes` | flag | Non-interactive mode |
| `--no-wait` | flag | Return the transition id immediately; don't poll for completion |
| `-o/--output` | string | Output format: `table`, `json` |

### power reset group \<GROUP_NAME\> / power reset nodes \<VALUE\>

Power-cycle nodes.

| Arg/Flag | Type | Description |
|----------|------|-------------|
| `-g/--graceful` | flag | Graceful reset |
| `-r/--reason` | string | Reason (group only) |
| `-y/--assume-yes` | flag | Non-interactive mode |
| `--no-wait` | flag | Return the transition id immediately; don't poll for completion |
| `-o/--output` | string | Output format: `table`, `json` |

```
manta power off group compute --graceful --assume-yes
manta power on group compute
manta power reset nodes 'x3000c0s1b0n[0-3]' --graceful
# Fire-and-forget — useful when the operator wants to do other work
# while the cluster transitions and check back later:
manta power reset group compute --no-wait
```

---

## log \[VALUE\]

Stream CFS session logs. Also available as the alias `manta logs`.

| Arg/Flag | Type | Description |
|----------|------|-------------|
| `VALUE` | string | Session name, group name, xname, or NID (interactive picker if omitted) |
| `-t/--timestamps` | flag | Show timestamps in log output |

```
manta log my-session
manta log my-session --timestamps
```

---

## console

### console node \<XNAME\>

Open an interactive console to a node (identified by xname or NID).

```
manta console node x3000c0s1b0n0
```

### console target-ansible \<SESSION_NAME\>

Open an interactive shell in the Ansible target container of a running CFS session. Useful for debugging in-progress sessions.

```
manta console target-ansible my-session
```

---

## gen-autocomplete

Generate a shell autocompletion script. Print it to stdout, or write
it to a directory. The CLI guesses the shell from `$SHELL` if `-s` is
omitted.

| Flag | Type | Description |
|------|------|-------------|
| `-s/--shell` | string | Shell: `bash`, `zsh`, `fish` (auto-detected from `$SHELL` if omitted) |
| `-p/--path` | dir | Write the script to this directory; prints to stdout if omitted |

```
manta gen-autocomplete --shell zsh --path ~/.zsh/completions
manta gen-autocomplete --shell bash --path /etc/bash_completion.d
manta gen-autocomplete --shell fish --path ~/.config/fish/completions
```

---

## gen-man

Generate one man page (`.1` file) per subcommand from the running
binary's clap tree and write them into a directory. Useful when you
installed manta via the shell installer (or built from source) and
want `man manta`, `man manta-apply-sat-file`, etc. to work locally
without juggling source-tree paths.

| Flag | Type | Description |
|------|------|-------------|
| `-p/--path` | dir | Target directory; defaults to `$XDG_DATA_HOME/man/man1` (i.e. `~/.local/share/man/man1`) when omitted |
| `-o/--output {table,json}` | enum | Format the result message (default `table`) |

```
manta gen-man
manta gen-man --path ~/.local/share/man/man1
manta gen-man --output json
```

On Linux the default user man directory (`~/.local/share/man`) is
searched by `man` automatically. On macOS you may need to add it to
`MANPATH`:

```bash
export MANPATH="$HOME/.local/share/man:$MANPATH"
```

---

## upgrade

Replace this `manta` binary with the latest release. Fetches the
highest `manta-cli-v*` tag from <https://github.com/eth-cscs/manta/releases>,
compares against the running version, downloads the right
platform tarball, and atomically swaps the binary in place. The
manta-server is untouched — server installs are infrastructure-managed.

| Flag | Description |
|------|-------------|
| `-c/--check` | Check for a newer version and print it, but don't apply |
| `-d/--dry-run` | Show what would happen without downloading or replacing |
| `-y/--assume-yes` | Skip the confirmation prompt |
| `-o/--output {table,json}` | Format the version-info output (default is a few human-readable lines; `json` emits the standard `{"status":"ok","message":"…","data":…}` envelope) |

```
manta upgrade --check
manta upgrade --dry-run
manta upgrade -y --output json
manta upgrade
```

Default output looks like:

```
Already up to date (v2.0.0-beta.30).
  current: v2.0.0-beta.30
  latest:  v2.0.0-beta.30
  target:  aarch64-apple-darwin
```

> If you installed manta via Homebrew, prefer `brew upgrade manta-cli` —
> `manta upgrade` will warn (but not block) when it detects a
> Homebrew-managed install path.

---

## Node expressions

Most commands that accept node lists support three formats interchangeably:

| Format | Example |
|--------|---------|
| Single xname | `x3000c0s1b0n0` |
| Single NID | `nid001313` |
| Comma-separated | `x3000c0s1b0n0,x3000c0s1b0n1` |
| Hostlist expression | `x3000c0s1b0n[0-3]`, `nid00131[0-9]` |
