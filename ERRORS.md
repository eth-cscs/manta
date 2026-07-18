# Manta error codes

Every failure manta reports carries a stable, machine-readable
`MANTA_*` code next to its human-readable message:

- **HTTP API** — the JSON error body's `code` field:
  `{ "error": "session foo not found", "code": "MANTA_SESSION_NOT_FOUND" }`
- **CLI** — in brackets before the message:
  `HTTP 404 [MANTA_SESSION_NOT_FOUND]: session foo not found`
  (CLI-local failures use the same shape without the HTTP prefix:
  `[MANTA_SERVER_UNREACHABLE] cannot reach manta server at …`)
- **Server logs** — in the handler-boundary log lines:
  `Service error 404 Not Found [MANTA_SESSION_NOT_FOUND]: …`

Reference the **code** in scripts, bug reports, and support requests.

## Stability policy

- Codes are **append-only**: never renamed, reused, or removed. An
  obsolete code is marked *deprecated* here, not deleted.
- **Messages are not stable.** Wording may change between releases;
  only the code is a reliable key.
- One code identifies a failure *class*, not a call site — the same
  code can come from different commands, with the specifics in the
  message.
- The `code` field is optional on the wire: a server or CLI from
  before its introduction simply omits it.

The catalog lives in
`crates/manta-shared/src/common/error_code.rs`; a unit test
(`every_code_is_documented`) fails if a code exists there without a
row in this file.

## Catalog

"HTTP" is the status the manta-server API typically pairs with the
code ("—" = the failure happens CLI-side, before or instead of an
HTTP response). `MANTA_BACKEND_HTTP_ERROR` forwards the upstream
backend's own status.

### Invalid input

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_BAD_REQUEST` | 400 | The request failed validation. The message names the offending parameter or field — fix the command arguments or request body. |
| `MANTA_INVALID_PATTERN` | 400 | A pattern (hardware pattern, hostlist expression, glob) didn't parse. Check the expression syntax. |
| `MANTA_INVALID_NODE_ID` | 400 | A node id (xname) is invalid. Check the xname spelling/format. |
| `MANTA_INVALID_DATETIME` | 400 | A datetime argument didn't parse. Use the format shown in the message (e.g. `2026-07-17T12:00:00`). |
| `MANTA_UNSUPPORTED_BACKEND` | 400 | The operation isn't available for this site's backend type (CSM vs OpenCHAMI). |

### Authentication and authorisation

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_AUTH_TOKEN_NOT_FOUND` | 401 | No usable auth token on the request (missing/expired/non-Bearer). Re-authenticate (`manta auth login`). |
| `MANTA_JWT_MALFORMED` | 401 | The token is structurally invalid. Re-authenticate to obtain a fresh one. |
| `MANTA_INVALID_CREDENTIALS` | 401 | The backend evaluated and rejected the username/password. Details deliberately stay server-side. A backend that was *unreachable* during login returns a `MANTA_BACKEND_*` gateway error instead — unless the failure arrived pre-stringified from the backend crates (see the note under [Upstream backend](#upstream-backend-csm--openchami)), in which case it still falls back to this code. |
| `MANTA_FORBIDDEN` | 403 | The token is valid but a role forbids the operation (e.g. the read-only role on a mutating endpoint). |
| `MANTA_RATE_LIMITED` | 429 | Too many authentication attempts from this source. Wait and retry. |

### Resource state

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_NOT_FOUND` | 404 | Generic lookup failure; the message names the resource. Check the name/id. |
| `MANTA_SESSION_NOT_FOUND` | 404 | CFS session not found. |
| `MANTA_CONFIGURATION_NOT_FOUND` | 404 | CFS configuration not found. |
| `MANTA_SITE_NOT_FOUND` | 404 / — | The requested site isn't hosted by this manta-server. Check `site` in `cli.toml` / the `X-Manta-Site` header. |
| `MANTA_CONFLICT` | 409 | The operation conflicts with current resource state. |
| `MANTA_CONFIGURATION_ALREADY_EXISTS` | 409 | A configuration with that name already exists. |
| `MANTA_INSUFFICIENT_RESOURCES` | 422 | Not enough capacity/nodes to satisfy the request. |
| `MANTA_NOT_CONFIGURED` | 501 | The feature needs per-site Vault / Kubernetes settings the server doesn't have (see README's server configuration). |

### Upstream backend (CSM / OpenCHAMI)

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_BACKEND_HTTP_ERROR` | upstream | The backend returned an HTTP error; manta forwards its status, and the message carries the upstream detail. (On `POST /auth/token`, backend 5xx surfaces as 502 with this code.) |
| `MANTA_BACKEND_TIMEOUT` | 504 | The manta-server → backend call timed out. The backend may still be processing; check backend health. |
| `MANTA_BACKEND_CONNECT_FAILED` | 500 / 502 | manta-server couldn't open a TCP/TLS connection to the backend. Check the site's backend URL and reachability. |
| `MANTA_NETWORK_ERROR` | 500 / 502 | Other outbound HTTP failure (DNS, body stream, protocol). |

> **Known limitation:** many upstream failures currently arrive from the
> backend crates as pre-formatted strings rather than typed errors, and
> therefore surface as `MANTA_INTERNAL` with the network detail only in
> the message (e.g. a dead CSM on ordinary endpoints). The codes above
> fire where the error arrives typed — notably the login exchange and
> the server's own outbound timeouts. Finer, reliably-typed
> classification is tracked in
> [#107](https://github.com/eth-cscs/manta/issues/107).

### Internal

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_MISSING_FIELD` | 500 | A required field was absent in backend data or configuration. Usually a backend-data or server-config problem — report it with the message. |
| `MANTA_IO_ERROR` | 500 | Filesystem I/O failure (config file, token cache, …). |
| `MANTA_SERDE_ERROR` | 500 | JSON (de)serialisation failure. |
| `MANTA_YAML_ERROR` | 500 | YAML parse/serialise failure (SAT files). |
| `MANTA_TOML_ERROR` | 500 | TOML parse/edit failure (config files). |
| `MANTA_CONFIG_ERROR` | 500 / — | Configuration loading/schema failure (`cli.toml` / `server.toml`). Fix the file named in the message. |
| `MANTA_KAFKA_ERROR` | 500 | Kafka (audit) producer failure. |
| `MANTA_K8S_ERROR` | 500 | Kubernetes API interaction failed. |
| `MANTA_CONSOLE_ERROR` | 500 | Node-console session failure. |
| `MANTA_TEMPLATE_ERROR` | 500 / — | Jinja2/template rendering failed (SAT files). Fix the template named in the message. |
| `MANTA_HOOK_ERROR` | 500 | A configured hook failed. |
| `MANTA_GIT_ERROR` | 500 | A local git operation failed. |
| `MANTA_INTERNAL` | 500 | Unclassified internal error — report it with the full message. |

### CLI ↔ manta-server transport (CLI-side only)

| Code | HTTP | Meaning / what to do |
|------|------|----------------------|
| `MANTA_SERVER_UNREACHABLE` | — | The CLI couldn't connect to manta-server (refused, unreachable, connect timeout). Check `manta_server_url` in `cli.toml` and that the server is running. |
| `MANTA_SERVER_TIMEOUT` | — | The CLI-side request timeout fired (`request_timeout_secs` in `cli.toml`, default 300 s). The server may still be working; raise the timeout for heavy calls. |
| `MANTA_TRANSPORT_ERROR` | — | Other CLI ↔ manta-server transport failure; the message carries the underlying cause. |
