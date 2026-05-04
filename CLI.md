# Manta CLI Reference

Complete reference for all `manta` commands, subcommands, and flags.

**Global flag** (available on every command):

| Flag | Description |
|------|-------------|
| `--site <SITE_NAME>` | Override the active site from config for this invocation |

---

## serve

Run manta as an HTTPS REST + WebSocket API server.

```
manta serve --cert <cert.pem> --key <key.pem> [--port 8443] [--listen-address 0.0.0.0]
```

| Flag | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `--cert` | path | **yes** | — | TLS certificate PEM file |
| `--key` | path | **yes** | — | TLS private key PEM file |
| `--port` | u16 | no | `8443` | Port to listen on |
| `--listen-address` | string | no | `0.0.0.0` | Bind address |

See [API.md](API.md) for the full HTTP endpoint reference.

---

## config

Manage the local manta configuration file (`~/.config/manta/config.toml`).

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

### config gen-autocomplete

Generate a shell autocompletion script.

| Flag | Type | Description |
|------|------|-------------|
| `-s/--shell` | string | Shell: `bash`, `zsh`, `fish` (auto-detected from `$SHELL` if omitted) |
| `-p/--path` | dir | Write the script to this directory; prints to stdout if omitted |

```
manta config gen-autocomplete --shell zsh --path ~/.zsh/completions
```

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

### get hardware cluster \<CLUSTER_NAME\>

List hardware components in a cluster.

| Arg/Flag | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `CLUSTER_NAME` | string | **yes** | — | Cluster (HSM group) name |
| `-o/--output` | string | no | `summary` | Output format: `json`, `summary`, `details` |

```
manta get hardware cluster gpu-cluster -o details
```

### get hardware node \<XNAMES\>

List hardware components for specific nodes.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `XNAMES` | string | **yes** | Comma-separated xnames |
| `-t/--type` | string | no | Filter to a specific hardware artifact type |
| `-o/--output` | string | no | Output format: `json` |

```
manta get hardware node x3000c0s1b0n0,x3000c0s1b0n1
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
| `-H/--hsm-group` | string | HSM group name to filter by |
| `-o/--output` | string | Output format: `json` |

> `--hsm-group`, `--xnames`, and `--name` are mutually exclusive.  
> `--most-recent` and `--limit` are mutually exclusive.

```
manta get sessions --hsm-group compute --status running
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
| `-H/--hsm-group` | string | HSM group to filter by |
| `-o/--output` | string | Output format: `json` |

> `--hsm-group` and `--name` are mutually exclusive.  
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
| `-H/--hsm-group` | string | — | HSM group name |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### get cluster \<HSM_GROUP_NAME\>

Show node membership and status for a cluster.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `HSM_GROUP_NAME` | string | — | HSM group name |
| `-n/--nids-only-one-line` | flag | — | Print NIDs as a single comma-separated line |
| `-x/--xnames-only-one-line` | flag | — | Print xnames as a single comma-separated line |
| `-s/--status` | string | — | Filter by status: `OFF`, `ON`, `READY`, `STANDBY`, `PENDING`, `FAILED`, `CONFIGURED` |
| `-T/--summary-status` | flag | — | Print overall status summary |
| `-o/--output` | string | `table` | Output format: `table`, `table-wide`, `json`, `summary` |

```
manta get cluster compute -o summary
manta get cluster compute --xnames-only-one-line
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
manta get nodes x3000c0s1b0n[0-3]
manta get nodes nid001313,nid001314 -o json
```

### get images

List IMS images.

| Flag | Type | Description |
|------|------|-------------|
| `-i/--id` | string | Specific image ID |
| `-m/--most-recent` | flag | Show most recent only |
| `-l/--limit` | u8 | Return last N images |
| `-H/--hsm-group` | string | HSM group name |

### get boot-parameters

Get BSS boot parameters.

| Flag | Type | Description |
|------|------|-------------|
| `-H/--hsm-group` | string | HSM group name |
| `-n/--nodes` | string | Comma-separated xnames/nids or hostlist expression |

> One of `--hsm-group` or `--nodes` is required.

```
manta get boot-parameters --hsm-group compute
```

### get kernel-parameters

Get kernel boot parameters for nodes.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-n/--nodes` | string | — | Comma-separated xnames/nids or hostlist |
| `-H/--hsm-group` | string | — | HSM group name |
| `-f/--filter` | string | — | Comma-separated parameter names to extract, e.g. `console,crashkernel` |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

> One of `--nodes` or `--hsm-group` is required.

```
manta get kernel-parameters --hsm-group compute --filter console,loglevel
```

### get redfish-endpoints

List registered Redfish endpoints (BMCs).

| Flag | Type | Description |
|------|------|-------------|
| `-i/--id` | string | Filter by xname (repeatable) |
| `-f/--fqdn` | string | Filter by FQDN |
| `-u/--uuid` | string | Filter by UUID |
| `-m/--macaddr` | string | Filter by MAC address |
| `-I/--ipaddress` | string | Filter by IP address |

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
manta add group --label my-cluster --description "GPU nodes" --nodes x3000c0s1b0n[0-7]
```

### add node

Register a node with HSM.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--id` | string | **yes** | Node xname |
| `-g/--group` | string | **yes** | Group the node belongs to |
| `-H/--hardware` | path | no | File with hardware info |
| `-a/--arch` | string | no | Architecture: `X86`, `ARM`, `Other` |
| `-d/--disabled` | flag | no | Disable node on creation |

### add hardware *(WIP)*

Add hardware components to a cluster.

| Flag | Type | Description |
|------|------|-------------|
| `-P/--pattern` | string | Hardware pattern, e.g. `a100:4:epyc:2` |
| `-t/--target-cluster` | string | Target cluster name |
| `-p/--parent-cluster` | string | Source cluster providing the hardware |
| `-d/--dry-run` | flag | Simulate without changes |
| `-c/--create-hsm-group` | flag | Create target HSM group if it does not exist |

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
| `-H/--hsm-group` | string | no* | Target HSM group |
| `-O/--overwrite` | flag | no | Overwrite existing values |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip node reboot after change |
| `-d/--dry-run` | flag | no | Simulate without changes |

> One of `--nodes` or `--hsm-group` is required.

```
manta add kernel-parameters "console=ttyS0,115200 loglevel=7" --hsm-group compute
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

## update

Modify existing resources.

### update boot-parameters

Update existing BSS boot parameters for nodes.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-H/--hosts` | string | **yes** | Comma-separated xnames |
| `-p/--params` | string | no | Kernel parameters |
| `-k/--kernel` | string | no | S3 path to kernel file |
| `-i/--initrd` | string | no | S3 path to initrd file |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-y/--assume-yes` | flag | no | Non-interactive mode |

### update redfish-endpoint

Update an existing Redfish endpoint. Accepts the same flags as [`add redfish-endpoint`](#add-redfish-endpoint); `-i/--id` is required to identify the entry.

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

### apply session

Create a CFS session from one or more local git repositories.

| Flag | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `-n/--name` | string | **yes** | — | Session name |
| `-r/--repo-path` | path… | **yes** | — | Path(s) to local git repos (repeatable) |
| `-l/--ansible-limit` | string | no* | — | Target xnames |
| `-H/--hsm-group` | string | no* | — | HSM group scope |
| `-p/--playbook-name` | string | no | `site.yml` | Ansible playbook filename |
| `-w/--watch-logs` | flag | no | — | Stream session logs |
| `-t/--timestamps` | flag | no | — | Show timestamps in logs |
| `-v/--ansible-verbosity` | 0–4 | no | `2` | Ansible verbosity |
| `-P/--ansible-passthrough` | string | no | — | Extra Ansible parameters |

> One of `--ansible-limit` or `--hsm-group` is required.

```
manta apply session -n my-session -r ~/repos/csm-config -l x3000c0s1b0n0
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

### apply boot cluster \<CLUSTER_NAME\>

Update boot parameters (image + runtime config + kernel parameters) for all nodes in a cluster.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `CLUSTER_NAME` | string | **yes** | Cluster (HSM group) name |
| `-i/--boot-image` | string | no* | Specific IMS image ID |
| `-b/--boot-image-configuration` | string | no* | CFS configuration name to derive boot image from |
| `-r/--runtime-configuration` | string | no | CFS configuration for post-boot configuration |
| `-k/--kernel-parameters` | string | no | Kernel parameters string |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Update boot params without rebooting |
| `-d/--dry-run` | flag | no | Simulate without changes |

> `--boot-image` and `--boot-image-configuration` are mutually exclusive.

```
manta apply boot cluster compute -b csm-config-2024 -r csm-config-2024
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

### apply kernel-parameters \<VALUE\>

Replace kernel parameters for nodes (full replace, not merge).

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Space-separated `key=value` pairs |
| `-n/--nodes` | string | no* | Target nodes (xnames/nids/hostlist) |
| `-H/--hsm-group` | string | no* | Target HSM group |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip reboot |
| `-d/--dry-run` | flag | no | Simulate without changes |

> To merge parameters instead of replacing, use [`add kernel-parameters`](#add-kernel-parameters-value).

### apply ephemeral-environment

Launch a temporary container environment from an IMS image, useful for image inspection and debugging.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--image-id` | string | **yes** | IMS image ID to use as the container image |

### apply hardware cluster *(WIP)*

Upscale or downscale hardware resources in a cluster.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-P/--pattern` | string | **yes** | Hardware pattern e.g. `a100:12:epyc:5` |
| `-t/--target-cluster` | string | **yes** | Cluster being reconfigured |
| `-p/--parent-cluster` | string | **yes** | Cluster offering or receiving freed resources |
| `-d/--dry-run` | flag | no | Simulate without changes |
| `-c/--create-target-hsm-group` | flag | no | Create target cluster if it does not exist |
| `-D/--delete-empty-parent-hsm-group` | flag | no | Delete parent cluster if it becomes empty |
| `-u/--unpin-nodes` | flag | no | Allow any available nodes instead of pinned ones |

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

Remove a node from HSM.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `VALUE` | string | **yes** | Node xname |

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
| `-H/--hsm-group` | string | no* | Target HSM group |
| `-y/--assume-yes` | flag | no | Non-interactive mode |
| `--do-not-reboot` | flag | no | Skip reboot |
| `-d/--dry-run` | flag | no | Simulate without changes |

> One of `--nodes` or `--hsm-group` is required.

### delete boot-parameters

Delete BSS boot parameters for nodes.

| Flag | Type | Description |
|------|------|-------------|
| `-H/--hosts` | string | Comma-separated xnames |

### delete hardware *(WIP)*

Remove hardware components from a cluster, returning them to the parent pool.

| Flag | Type | Description |
|------|------|-------------|
| `-P/--pattern` | string | Hardware pattern |
| `-t/--target-cluster` | string | Cluster losing resources |
| `-p/--parent-cluster` | string | Cluster receiving freed resources |
| `-d/--dry-run` | flag | Simulate without changes |
| `-D/--delete-hsm-group` | flag | Delete the target cluster if it becomes empty |

### delete redfish-endpoint

Delete a registered Redfish endpoint.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-i/--id` | string | **yes** | Xname of the endpoint |

---

## migrate

Backup and restore virtual cluster configurations, or move nodes.

### migrate vCluster backup

Backup a virtual cluster's full configuration (BOS, CFS, IMS image, HSM group).

| Flag | Type | Description |
|------|------|-------------|
| `-b/--bos` | string | BOS session template name |
| `-d/--destination` | dir | Destination folder for backup files |
| `-p/--pre-hook` | string | Shell command to run before backup |
| `-a/--post-hook` | string | Shell command to run after backup |

### migrate vCluster restore

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

### migrate nodes \<XNAMES\>

Move compute nodes from one HSM group to another.

| Arg/Flag | Type | Required | Description |
|----------|------|----------|-------------|
| `XNAMES` | string | **yes** | Comma-separated xnames to move |
| `-t/--to` | string | **yes** | Destination HSM group |
| `-f/--from` | string | no | Source HSM group |
| `-d/--dry-run` | flag | no | Simulate without changes |

```
manta migrate nodes x3000c0s1b0n[0-3] --to gpu-cluster --from nodes_free
```

---

## power

Manage node power state.

### power on cluster \<CLUSTER_NAME\>

Power on all nodes in a cluster.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `CLUSTER_NAME` | string | — | Cluster (HSM group) name |
| `-R/--reason` | string | — | Reason for the power action |
| `-y/--assume-yes` | flag | — | Non-interactive mode |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### power on nodes \<VALUE\>

Power on specific nodes.

| Arg/Flag | Type | Default | Description |
|----------|------|---------|-------------|
| `VALUE` | string | — | Xnames/nids or hostlist expression |
| `-y/--assume-yes` | flag | — | Non-interactive mode |
| `-o/--output` | string | `table` | Output format: `table`, `json` |

### power off cluster \<CLUSTER_NAME\> / power off nodes \<VALUE\>

Power off nodes.

| Arg/Flag | Type | Description |
|----------|------|-------------|
| `-g/--graceful` | flag | Graceful shutdown |
| `-R/--reason` | string | Reason for power-off (cluster only) |
| `-y/--assume-yes` | flag | Non-interactive mode |
| `-o/--output` | string | Output format: `table`, `json` |

### power reset cluster \<CLUSTER_NAME\> / power reset nodes \<VALUE\>

Power-cycle nodes.

| Arg/Flag | Type | Description |
|----------|------|-------------|
| `-g/--graceful` | flag | Graceful reset |
| `-r/--reason` | string | Reason (cluster only) |
| `-y/--assume-yes` | flag | Non-interactive mode |
| `-o/--output` | string | Output format: `table`, `json` |

```
manta power off cluster compute --graceful --assume-yes
manta power on cluster compute
manta power reset nodes x3000c0s1b0n[0-3] --graceful
```

---

## log \[VALUE\]

Stream CFS session logs.

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

## validate-local-repo

Verify that a local git repository's tags and HEAD commit exist in the configured Gitea instance. Useful before creating a CFS session.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `-r/--repo-path` | path… | **yes** | Path(s) to local git repos (repeatable) |

```
manta validate-local-repo -r ~/repos/csm-config
```

---

## add-nodes-to-groups

Add nodes to one or more HSM groups.

| Flag | Type | Description |
|------|------|-------------|
| `-g/--group` | string | Target HSM group |
| `-n/--nodes` | string | Nodes to add (xnames/nids/hostlist) |
| `-d/--dry-run` | flag | Simulate without changes |

---

## remove-nodes-from-groups

Remove nodes from one or more HSM groups.

| Flag | Type | Description |
|------|------|-------------|
| `-g/--group` | string | Source HSM group |
| `-n/--nodes` | string | Nodes to remove (xnames/nids/hostlist) |
| `-d/--dry-run` | flag | Simulate without changes |

---

## Node expressions

Most commands that accept node lists support three formats interchangeably:

| Format | Example |
|--------|---------|
| Single xname | `x3000c0s1b0n0` |
| Single NID | `nid001313` |
| Comma-separated | `x3000c0s1b0n0,x3000c0s1b0n1` |
| Hostlist expression | `x3000c0s1b0n[0-3]`, `nid00131[0-9]` |
