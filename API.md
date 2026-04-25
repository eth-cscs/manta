# Manta HTTP API Reference

The manta HTTP server exposes a REST API over HTTPS on port `8443` by default.

## Starting the server

```
manta serve --cert <cert.pem> --key <key.pem> [--port 8443] [--listen-addr 0.0.0.0]
```

## Authentication

Every endpoint (except `/health`) requires a Bearer token in the `Authorization` header:

```
Authorization: Bearer <shasta-token>
```

## Base URL

```
https://<host>:8443/api/v1
```

## Error responses

All errors return JSON with an `error` field:

```json
{ "error": "description of what went wrong" }
```

| Status | Meaning |
|--------|---------|
| `400` | Bad request — invalid parameters or body |
| `401` | Missing or malformed `Authorization` header |
| `404` | Resource not found |
| `409` | Conflict — resource already exists |
| `422` | Unprocessable entity — required field missing or wrong type |
| `500` | Backend call failed |
| `501` | Feature requires server config not set (`vault_base_url`, `k8s_api_url`) |

---

## Sessions

### GET /sessions

List CFS sessions, optionally filtered.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hsm_group` | string | no | Filter by HSM group name |
| `xnames` | string | no | Comma-separated xnames to filter by |
| `min_age` | string | no | Minimum session age (e.g. `1h`, `2d`) |
| `max_age` | string | no | Maximum session age |
| `session_type` | string | no | `image` or `node` |
| `status` | string | no | `pending`, `running`, `complete` |
| `name` | string | no | Exact session name |
| `limit` | u8 | no | Maximum number of results |

**Response `200`** — array of CFS session objects.

---

### POST /sessions/apply

Create a CFS configuration and session with HSM membership validation.

Identical to `POST /sessions` but also validates that every node in `ansible_limit` belongs to an HSM group the token has access to before creating the session.

> Requires `vault_base_url` configured on the server.

**Request body**

```json
{
  "repo_names": ["csm-config"],
  "repo_last_commit_ids": ["abc123def456"],
  "cfs_conf_sess_name": "my-session",
  "playbook_yaml_file_name": "site.yaml",
  "hsm_group": "compute",
  "ansible_limit": "x3000c0s1b0n0,x3000c0s2b0n0",
  "ansible_verbosity": "1",
  "ansible_passthrough": ""
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `repo_names` | string[] | **yes** | Gitea repository names |
| `repo_last_commit_ids` | string[] | **yes** | Commit SHA for each repo (same order as `repo_names`) |
| `cfs_conf_sess_name` | string | no | Name for the config and session (auto-generated if omitted) |
| `playbook_yaml_file_name` | string | no | Ansible playbook file (default: `site.yaml`) |
| `hsm_group` | string | no | Target HSM group |
| `ansible_limit` | string | no | Comma-separated xnames to limit execution; each must belong to an accessible HSM group |
| `ansible_verbosity` | string | no | Ansible verbosity level (`0`–`4`) |
| `ansible_passthrough` | string | no | Extra arguments passed to `ansible-playbook` |

**Response `201`**

```json
{
  "session_name": "my-session-20240101",
  "configuration_name": "my-session-20240101-config"
}
```

---

### POST /sessions

Create a CFS configuration and session from one or more git repositories.

> Requires `vault_base_url` configured on the server (used to fetch the Gitea token from Vault).

**Request body**

```json
{
  "repo_names": ["csm-config"],
  "repo_last_commit_ids": ["abc123def456"],
  "cfs_conf_sess_name": "my-session",
  "playbook_yaml_file_name": "site.yaml",
  "hsm_group": "compute",
  "ansible_limit": "x3000c0s1b0n0",
  "ansible_verbosity": "1",
  "ansible_passthrough": ""
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `repo_names` | string[] | **yes** | Gitea repository names |
| `repo_last_commit_ids` | string[] | **yes** | Commit SHA for each repo (same order) |
| `cfs_conf_sess_name` | string | no | Name for the config and session (auto-generated if omitted) |
| `playbook_yaml_file_name` | string | no | Ansible playbook file (default: `site.yaml`) |
| `hsm_group` | string | no | Target HSM group |
| `ansible_limit` | string | no | Comma-separated xnames or group names to limit execution |
| `ansible_verbosity` | string | no | Ansible verbosity level (`0`–`4`) |
| `ansible_passthrough` | string | no | Extra arguments passed to `ansible-playbook` |

**Response `201`**

```json
{
  "session_name": "my-session-20240101",
  "configuration_name": "my-session-20240101-config"
}
```

---

### DELETE /sessions/{name}

Delete and cancel a CFS session.

**Path parameters:** `name` — session name.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `dry_run` | bool | no | If `true`, return what would be deleted without deleting (default: `false`) |

**Response `200`** — on dry run: deletion context object. On delete: `{ "deleted": "<name>" }`.

---

### GET /sessions/{name}/logs

Stream CFS session logs as [Server-Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events).

> Requires `k8s_api_url` and `vault_base_url` configured on the server.

**Path parameters:** `name` — CFS session name.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `timestamps` | bool | no | Include timestamps in log lines (default: `false`) |

**Response `200`** — `Content-Type: text/event-stream`. Each log line is delivered as an SSE `data:` event.

```
curl --no-buffer -H "Authorization: Bearer $TOKEN" \
  https://host:8443/api/v1/sessions/my-session/logs
```

---

## Configurations

### GET /configurations

List CFS configurations, optionally filtered.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | no | Exact configuration name |
| `pattern` | string | no | Name pattern (glob) |
| `hsm_group` | string | no | Filter by associated HSM group |
| `limit` | u8 | no | Maximum number of results |

**Response `200`** — array of CFS configuration objects.

---

### DELETE /configurations

Delete CFS configurations and their dependent images and session templates.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pattern` | string | no | Name pattern to match configurations |
| `since` | string | no | Delete configurations created after this datetime (`YYYY-MM-DDTHH:MM:SS`) |
| `until` | string | no | Delete configurations created before this datetime |
| `dry_run` | bool | no | Preview without deleting (default: `false`) |

**Response `200`**

```json
{
  "deleted_configurations": ["config-a", "config-b"],
  "deleted_images": ["img-uuid-1"]
}
```

---

## Nodes

### GET /nodes

Get details for one or more nodes.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `xname` | string | **yes** | Node xname (e.g. `x3000c0s1b0n0`) |
| `include_siblings` | bool | no | Include sibling nodes in the same blade (default: `false`) |
| `status` | string | no | Filter by power status |

**Response `200`** — array of node objects.

---

### POST /nodes

Register a new node.

**Request body**

```json
{
  "id": "x3000c0s1b0n0",
  "group": "compute",
  "enabled": true,
  "arch": "X86"
}
```

| Field | Type | Required |
|-------|------|----------|
| `id` | string | **yes** |
| `group` | string | **yes** |
| `enabled` | bool | no (default: `false`) |
| `arch` | string | no |

**Response `201`** — `{ "id": "<xname>" }`.

---

### DELETE /nodes/{id}

Delete a node by xname.

**Path parameters:** `id` — node xname.

**Response `204`** — no content.

---

## Groups (HSM)

### GET /groups

List HSM groups.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | no | Exact group name |

**Response `200`** — array of HSM group objects.

---

### POST /groups

Create a new HSM group.

**Request body** — HSM group object:

```json
{
  "label": "my-group",
  "description": "My compute nodes",
  "members": { "ids": ["x3000c0s1b0n0", "x3000c0s3b0n0"] }
}
```

**Response `201`** — no body.

---

### DELETE /groups/{label}

Delete an HSM group.

**Path parameters:** `label` — group label.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `force` | bool | no | Skip orphan-node check (default: `false`) |

**Response `204`** — no content.

---

### POST /groups/{name}/members

Add nodes to an HSM group.

**Path parameters:** `name` — group name.

**Request body**

```json
{ "hosts_expression": "x3000c0s[1-4]b0n0" }
```

**Response `200`**

```json
{
  "added": ["x3000c0s1b0n0", "x3000c0s2b0n0"],
  "removed": []
}
```

---

### DELETE /groups/{name}/members

Remove nodes from an HSM group.

**Path parameters:** `name` — group name.

**Request body**

```json
{
  "xnames": ["x3000c0s1b0n0", "x3000c0s2b0n0"],
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `xnames` | string[] | **yes** | Nodes to remove |
| `dry_run` | bool | no | Preview without removing (default: `false`) |

**Response `204`** — no content.

---

## Templates (BOS)

### GET /templates

List BOS session templates.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | no | Exact template name |
| `hsm_group` | string | no | Filter by associated HSM group |
| `limit` | u8 | no | Maximum number of results |

**Response `200`** — array of BOS session template objects.

---

### POST /templates/{name}/sessions

Create a BOS session from a named template.

**Path parameters:** `name` — BOS session template name.

**Request body**

```json
{
  "operation": "reboot",
  "limit": "compute",
  "session_name": "my-reboot",
  "include_disabled": false,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operation` | string | **yes** | `boot`, `reboot`, or `shutdown` |
| `limit` | string | **yes** | Comma-separated xnames or HSM group names |
| `session_name` | string | no | Name for the BOS session (auto-generated if omitted) |
| `include_disabled` | bool | no | Include disabled nodes (default: `false`) |
| `dry_run` | bool | no | Return the session object without creating it (default: `false`) |

**Response `201`** (or `200` on dry run) — BOS session object.

---

## Images

### GET /images

List IMS images.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | no | Exact image ID |
| `hsm_group` | string | no | Filter by associated HSM group |
| `limit` | u8 | no | Maximum number of results |

**Response `200`** — array of objects:

```json
[
  {
    "image": { ... },
    "configuration_name": "csm-config-1.0",
    "image_id": "uuid-here",
    "is_linked": true
  }
]
```

---

### DELETE /images

Delete one or more IMS images.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `ids` | string | **yes** | Comma-separated image IDs |
| `dry_run` | bool | no | Preview without deleting (default: `false`) |

**Response `200`**

```json
{ "deleted": ["uuid-1", "uuid-2"] }
```

---

## Boot parameters

### GET /boot-parameters

Get BSS boot parameters.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hsm_group` | string | no | Filter by HSM group |
| `nodes` | string | no | Comma-separated xnames |

**Response `200`** — boot parameters object.

---

### POST /boot-parameters

Add boot parameters.

**Request body** — BSS BootParameters object.

**Response `201`** — no body.

---

### PUT /boot-parameters

Update boot parameters.

**Request body** — updated BSS BootParameters object.

**Response `204`** — no content.

---

### DELETE /boot-parameters

Delete boot parameters for a set of nodes.

**Request body**

```json
{ "hosts": ["x3000c0s1b0n0"] }
```

**Response `204`** — no content.

---

## Kernel parameters

### GET /kernel-parameters

Get kernel parameters for nodes.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hsm_group` | string | no | Filter by HSM group |
| `nodes` | string | no | Comma-separated xnames |

**Response `200`** — kernel parameters object.

---

### POST /kernel-parameters/add

Append kernel parameters to a set of nodes without replacing existing ones.

**Request body**

```json
{
  "params": "console=ttyS0,115200n8",
  "xnames": ["x3000c0s1b0n0"],
  "hsm_group": null,
  "overwrite": false,
  "project_sbps": true,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `params` | string | **yes** | Space-separated kernel parameters to add |
| `xnames` | string[] | no | Target nodes (use instead of `hsm_group`) |
| `hsm_group` | string | no | Target HSM group (use instead of `xnames`) |
| `overwrite` | bool | no | Overwrite a parameter if it already exists (default: `false`) |
| `project_sbps` | bool | no | Project SBPS images (default: `true`) |
| `dry_run` | bool | no | Preview without persisting (default: `false`) |

**Response `200`**

```json
{
  "applied": true,
  "has_changes": true,
  "xnames_to_reboot": ["x3000c0s1b0n0"]
}
```

---

### POST /kernel-parameters/apply

Add, replace, or delete kernel parameters for a set of nodes.

**Request body**

```json
{
  "xnames": ["x3000c0s1b0n0"],
  "operation": "add",
  "params": "console=ttyS0,115200n8",
  "overwrite": false,
  "project_sbps": true,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `xnames` | string[] | **yes** | Target nodes |
| `operation` | string | **yes** | `add`, `apply` (replace all), or `delete` |
| `params` | string | **yes** | Space-separated kernel parameters |
| `overwrite` | bool | no | For `add`: overwrite existing params (default: `false`) |
| `project_sbps` | bool | no | Project SBPS images (default: `true`) |
| `dry_run` | bool | no | Preview without persisting (default: `false`) |

**Response `200`**

```json
{
  "applied": true,
  "has_changes": true,
  "xnames_to_reboot": ["x3000c0s1b0n0"]
}
```

---

### DELETE /kernel-parameters

Remove specific kernel parameters from a set of nodes.

**Request body**

```json
{
  "params": "console=ttyS0,115200n8",
  "xnames": ["x3000c0s1b0n0"],
  "hsm_group": null,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `params` | string | **yes** | Space-separated kernel parameters to remove |
| `xnames` | string[] | no | Target nodes (use instead of `hsm_group`) |
| `hsm_group` | string | no | Target HSM group (use instead of `xnames`) |
| `dry_run` | bool | no | Preview without persisting (default: `false`) |

**Response `200`**

```json
{
  "applied": true,
  "has_changes": true,
  "xnames_to_reboot": ["x3000c0s1b0n0"]
}
```

---

## Boot configuration

### POST /boot-config

Apply a combined boot configuration (image + runtime config + kernel params) to a set of nodes.

**Request body**

```json
{
  "hosts_expression": "compute",
  "boot_image_id": "ims-image-uuid",
  "boot_image_configuration": "csm-config-1.0",
  "kernel_parameters": "console=ttyS0",
  "runtime_configuration": "csm-config-1.0",
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `hosts_expression` | string | **yes** | Xnames, nodeset expression, or HSM group name |
| `boot_image_id` | string | no | IMS image ID to set as boot image |
| `boot_image_configuration` | string | no | CFS configuration to link to the boot image |
| `kernel_parameters` | string | no | Kernel parameters to set |
| `runtime_configuration` | string | no | CFS configuration for runtime |
| `dry_run` | bool | no | Preview without persisting (default: `false`) |

**Response `200`**

```json
{
  "applied": true,
  "nodes": ["x3000c0s1b0n0"],
  "need_restart": false
}
```

---

## Power management

### POST /power

Power on, off, or reset nodes or an entire cluster.

**Request body**

```json
{
  "action": "reboot",
  "targets": ["x3000c0s1b0n0", "x3000c0s3b0n0"],
  "target_type": "nodes",
  "force": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | string | **yes** | `on`, `off`, or `reset` |
| `targets` | string[] | **yes** | Xnames (for `target_type: nodes`) or `[group-name]` (for `target_type: cluster`) |
| `target_type` | string | **yes** | `nodes` or `cluster` |
| `force` | bool | no | Hard power off/reset without graceful shutdown (default: `false`) |

**Response `200`** — PCS `TransitionResponse` object.

---

## Redfish endpoints

### GET /redfish-endpoints

List Redfish endpoints.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | no | Filter by ID |
| `fqdn` | string | no | Filter by FQDN |
| `uuid` | string | no | Filter by UUID |
| `macaddr` | string | no | Filter by MAC address |
| `ipaddress` | string | no | Filter by IP address |

**Response `200`** — array of Redfish endpoint objects.

---

### POST /redfish-endpoints

Add a Redfish endpoint.

**Request body** — Redfish endpoint parameters object.

**Response `201`** — no body.

---

### PUT /redfish-endpoints

Update a Redfish endpoint.

**Request body** — Redfish endpoint parameters object.

**Response `204`** — no content.

---

### DELETE /redfish-endpoints/{id}

Delete a Redfish endpoint.

**Path parameters:** `id` — endpoint ID.

**Response `204`** — no content.

---

## Hardware inventory

### GET /clusters

Get cluster node details with optional status filtering.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hsm_group` | string | no | Filter by HSM group |
| `status` | string | no | Filter by node power status |

**Response `200`** — array of cluster node objects.

---

### GET /hardware-clusters

Get hardware component summary for one or more clusters.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hsm_group` | string | no | Filter by HSM group |

**Response `200`** — object with `hsm_group_name` and `node_summaries`.

---

### GET /hardware-nodes

Get hardware component details for specific nodes.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `xnames` | string | **yes** | Comma-separated xnames |
| `type_artifact` | string | no | Filter by component type |

**Response `200`** — hardware node summary object.

---

## Hardware cluster management

### POST /hardware-clusters/{target}/members

Add hardware components (nodes) to a target cluster, sourcing them from a parent cluster.

**Path parameters:** `target` — destination HSM group name.

**Request body**

```json
{
  "parent_cluster": "nodes_free",
  "pattern": "Memory=512:Processors=2",
  "create_hsm_group": false,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `parent_cluster` | string | **yes** | Source HSM group to draw nodes from |
| `pattern` | string | **yes** | Hardware component pattern to match (e.g. `Memory=512:Processors=2`) |
| `create_hsm_group` | bool | no | Create the target group if it doesn't exist (default: `false`) |
| `dry_run` | bool | no | Preview without moving nodes (default: `false`) |

**Response `200`**

```json
{
  "dry_run": false,
  "nodes_moved": ["x3000c0s1b0n0"],
  "target_cluster": "my-cluster",
  "target_nodes": ["x3000c0s1b0n0"],
  "parent_cluster": "nodes_free",
  "parent_nodes": ["x3000c0s2b0n0"]
}
```

---

### DELETE /hardware-clusters/{target}/members

Remove hardware components from a target cluster and return them to a parent cluster.

**Path parameters:** `target` — source HSM group name.

**Request body**

```json
{
  "parent_cluster": "nodes_free",
  "pattern": "Memory=512:Processors=2",
  "delete_hsm_group": false,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `parent_cluster` | string | **yes** | Destination HSM group for returned nodes |
| `pattern` | string | **yes** | Hardware component pattern to match |
| `delete_hsm_group` | bool | no | Delete the target group if it becomes empty (default: `false`) |
| `dry_run` | bool | no | Preview without moving nodes (default: `false`) |

**Response `200`** — same shape as `POST /hardware-clusters/{target}/members`.

---

### POST /hardware-clusters/{target}/configuration

Pin or unpin a hardware cluster configuration by moving nodes between the target and parent cluster according to a hardware component pattern.

**Path parameters:** `target` — target HSM group name.

**Request body**

```json
{
  "parent_cluster": "nodes_free",
  "pattern": "Memory=512:Processors=2",
  "mode": "pin",
  "create_target_hsm_group": true,
  "delete_empty_parent_hsm_group": true,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `parent_cluster` | string | **yes** | Source/destination HSM group |
| `pattern` | string | **yes** | Hardware component pattern to match |
| `mode` | string | no | `"pin"` (move nodes into target) or `"unpin"` (move nodes back to parent) — default: `"pin"` |
| `create_target_hsm_group` | bool | no | Create the target group if it doesn't exist (default: `true`) |
| `delete_empty_parent_hsm_group` | bool | no | Delete the parent group if it becomes empty (default: `true`) |
| `dry_run` | bool | no | Preview without moving nodes (default: `false`) |

**Response `200`**

```json
{
  "dry_run": false,
  "target_cluster": "my-cluster",
  "target_nodes": ["x3000c0s1b0n0"],
  "parent_cluster": "nodes_free",
  "parent_nodes": []
}
```

---

## Migration

### POST /migrate/nodes

Move nodes between HSM groups (vClusters).

**Request body**

```json
{
  "target_hsm_names": ["target-cluster"],
  "parent_hsm_names": ["parent-cluster"],
  "hosts_expression": "x3000c0s[1-4]b0n0",
  "dry_run": false,
  "create_hsm_group": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_hsm_names` | string[] | **yes** | Destination HSM group(s) |
| `parent_hsm_names` | string[] | **yes** | Source HSM group(s) |
| `hosts_expression` | string | **yes** | Nodes to migrate |
| `dry_run` | bool | no | Preview without migrating (default: `false`) |
| `create_hsm_group` | bool | no | Create the target group if it doesn't exist (default: `false`) |

**Response `200`** — migration results object.

---

### POST /migrate/backup

Back up vCluster configuration to files.

**Request body**

```json
{
  "bos": "my-cluster",
  "destination": "/backups/cluster"
}
```

**Response `200`** — `{ "status": "backup completed" }`.

---

### POST /migrate/restore

Restore a vCluster from backup files.

**Request body**

```json
{
  "bos_file": "/backups/bos.yaml",
  "cfs_file": "/backups/cfs.yaml",
  "hsm_file": "/backups/hsm.yaml",
  "ims_file": "/backups/ims.yaml",
  "image_dir": "/backups/images",
  "overwrite": false
}
```

All fields optional. `overwrite` defaults to `false`.

**Response `200`** — `{ "status": "restore completed" }`.

---

## Ephemeral environments

### POST /ephemeral-env

Create an ephemeral CFS environment from an existing image.

**Request body**

```json
{ "image_id": "ims-image-uuid" }
```

**Response `201`** — `{ "status": "ephemeral environment created" }`.

---

## SAT files

### POST /sat-file

Apply a SAT (Shasta Artifact Template) file. Renders Jinja2 templates, builds images, creates BOS session templates, and optionally reboots nodes.

> Requires `vault_base_url` and `k8s_api_url` configured on the server.

**Request body**

```json
{
  "sat_file_content": "schema: 1.0\nimages:\n  - ...",
  "values": { "version": "1.5.0", "site": "alps" },
  "values_file_content": null,
  "ansible_verbosity": 0,
  "ansible_passthrough": "",
  "reboot": false,
  "watch_logs": false,
  "timestamps": false,
  "image_only": false,
  "session_template_only": false,
  "overwrite": false,
  "dry_run": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sat_file_content` | string | **yes** | SAT YAML file content (may contain Jinja2 syntax) |
| `values` | object | no | Key-value pairs for Jinja2 template rendering |
| `values_file_content` | string | no | YAML values file content (alternative to `values`) |
| `ansible_verbosity` | u8 | no | Ansible verbosity level 0–4 |
| `ansible_passthrough` | string | no | Extra arguments passed to `ansible-playbook` |
| `reboot` | bool | no | Reboot nodes after applying session templates (default: `false`) |
| `watch_logs` | bool | no | Stream CFS session logs during apply (default: `false`) |
| `timestamps` | bool | no | Include timestamps in streamed logs (default: `false`) |
| `image_only` | bool | no | Process only the `images` section (default: `false`) |
| `session_template_only` | bool | no | Process only the `session_templates` section (default: `false`) |
| `overwrite` | bool | no | Overwrite existing configurations and images (default: `false`) |
| `dry_run` | bool | no | Render and validate without creating anything (default: `false`) |

**Response `200`** — `{ "status": "SAT file applied successfully" }`.

---

## Interactive consoles (WebSocket)

Both console endpoints use the standard WebSocket upgrade handshake. Authentication is checked during the HTTP upgrade — a missing or invalid `Authorization` header returns `401` before the WebSocket connection is established. The server closes the connection after **30 minutes of inactivity**.

Once connected, the WebSocket carries raw terminal I/O:
- **Text or binary frames sent by the client** are forwarded as stdin to the console.
- **Binary frames sent by the server** are stdout from the console.

```
wscat -H "Authorization: Bearer $TOKEN" \
  --connect wss://host:8443/api/v1/nodes/x3000c0s1b0n0/console
```

---

### WS /nodes/{xname}/console

Open an interactive console to a node.

> Requires `k8s_api_url` and `vault_base_url` configured on the server.

**Path parameters:** `xname` — node xname (e.g. `x3000c0s1b0n0`).

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `cols` | u16 | no | Terminal width in columns (default: `80`) |
| `rows` | u16 | no | Terminal height in rows (default: `24`) |

**Upgrade response `101`** — WebSocket connection established.

---

### WS /sessions/{name}/console

Open an interactive console to the Ansible container of a running image-type CFS session.

> Requires `k8s_api_url` and `vault_base_url` configured on the server. The session must exist, be of type `image`, and have status `running`. Returns `409` if those conditions are not met.

**Path parameters:** `name` — CFS session name.

**Query parameters**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `cols` | u16 | no | Terminal width in columns (default: `80`) |
| `rows` | u16 | no | Terminal height in rows (default: `24`) |

**Upgrade response `101`** — WebSocket connection established.

---

## Health check

### GET /health

Returns server health. Does not require authentication.

**Response `200`** — `{ "status": "ok" }`.

---

## Server configuration requirements

Some endpoints require optional fields to be set in the server configuration (`~/.config/manta/config.toml`):

| Config field | Required by |
|---|---|
| `vault_base_url` | `POST /sessions`, `POST /sessions/apply`, `GET /sessions/{name}/logs`, `POST /sat-file`, `WS /nodes/{xname}/console`, `WS /sessions/{name}/console` |
| `k8s_api_url` | `GET /sessions/{name}/logs`, `POST /sat-file`, `WS /nodes/{xname}/console`, `WS /sessions/{name}/console` |
