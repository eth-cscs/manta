# Analysis: Should `site_name` Be Added to HTTP API Requests?

**Scope**: Evaluate whether the manta HTTP server should accept a `site_name` parameter per request and route to the matching backend dynamically, given the assumption that a JWT token is valid for any configured site.

---

## JWT Validity: Gate Passes

There is a centralized Keycloak that issues JWT tokens. Each site's Keycloak is configured to accept tokens from the centralized issuer. A single JWT is therefore valid across all configured sites.

**How token validation works today**: The `BearerToken` Axum extractor only parses the `Authorization: Bearer <token>` header — no validation at the manta layer. Every handler passes the raw token string to `backend.validate_api_token()`, which dispatches to csm-rs or ochami-rs and presents the token to that site's backend API. Because site Keycloaks federate with the central one, the token is accepted at every site.

This confirms the assumption. Multi-site routing is viable at the authentication layer.

---

## What "Adding `site_name`" Actually Means

Adding `site_name` to the API is only useful in a **multi-site server** model where one running `manta serve` process handles requests for several backend sites simultaneously.

In the alternative **one-server-per-site** model, the site is implied by which server the client connects to. The CLI already reads `site = "alps"` from config; in HTTP mode it would read `manta_url = "https://manta.alps.example.com"` from the same config block and direct all requests there. No `site_name` in the API is needed.

The value of the multi-site model is operational: fewer processes to run, one TLS certificate, one URL. The cost is a non-trivial refactor described below.

---

## Network Topology

```
[CLI / caller]
      │
      │  (caller may need SOCKS5 to reach the manta server — irrelevant to server internals)
      ▼
[manta HTTP server]
      │
      │  (server needs SOCKS5 to reach the backend: CSM / OpenCHAMI APIs)
      ▼
[Backend: CSM / OpenCHAMI]
```

The two SOCKS5 legs are independent. Only the server-to-backend leg is at issue for multi-site.

---

## SOCKS5: Hard Architectural Constraint

`main.rs` sets the proxy before the tokio runtime starts:

```rust
// main.rs:73–83
unsafe {
    std::env::set_var("SOCKS5", socks_proxy);
}
// then: rt.block_on(run(...))
```

csm-rs reads this env var on every `reqwest::Client` build (106 call sites). This is process-global.

`std::env::set_var` is marked `unsafe` since Rust 1.66 precisely because it is unsound in a multi-threaded context. For a multi-site server, there is no way to set different per-site proxies via env var — the fix is to remove the env-var coupling entirely and thread `socks5_proxy: Option<String>` explicitly through `InfraContext` into csm-rs. This requires centralizing `reqwest::Client` construction in csm-rs.

This refactor is a **hard prerequisite** for multi-site support regardless of whether sites happen to use the same proxy today. The current call to `set_var` at startup must be removed before `manta serve` is safe to run with multiple sites in its config.

---

## `site_name` Touch Points

### Direct consumers requiring explicit `site_name`

| Location | Purpose |
|---|---|
| `handlers::create_session` | Vault path: `manta/data/{site_name}` |
| `handlers::get_session_logs` | Vault path: `manta/data/{site_name}` |
| `handlers::post_sat_file` | Vault path: `manta/data/{site_name}` |
| `handlers::apply_session` | Vault path: `manta/data/{site_name}` |
| `handlers::console_node_ws` | K8s pod console streaming — needs site's K8s credentials |
| `handlers::console_session_ws` | K8s CFS session console streaming — needs site's K8s credentials |
| `service::configuration` (×2) | Vault path construction |
| `service::sat_file` (×2) | Vault path construction |

### Implicit consumers (all 47 handlers)

Every handler receives an `InfraContext` whose `backend: &StaticBackendDispatcher` carries a fixed `base_url`. Switching sites is not a string change — it requires a different `StaticBackendDispatcher` instance pointing at a different URL with a different root certificate.

---

## `ServerState` Refactor Scope

Current (single-site):
```rust
pub struct ServerState {
    pub backend: StaticBackendDispatcher,
    pub site_name: String,
    pub shasta_base_url: String,
    pub shasta_root_cert: Vec<u8>,
    pub vault_base_url: Option<String>,
    pub gitea_base_url: Option<String>,
    pub k8s_api_url: Option<String>,
    pub console_inactivity_timeout: u64,
}
```

Required for multi-site:
```rust
pub struct SiteBackend {
    pub backend: StaticBackendDispatcher,
    pub shasta_base_url: String,
    pub shasta_root_cert: Vec<u8>,
    pub socks5_proxy: Option<String>,   // must be explicit, not env-var
    pub vault_base_url: Option<String>,
    pub gitea_base_url: Option<String>,
    pub k8s_api_url: Option<String>,
    pub k8s_auth: Option<K8sAuth>,      // needed by console WebSocket handlers
}

pub struct ServerState {
    pub sites: HashMap<String, SiteBackend>,
    pub default_site: String,
    pub console_inactivity_timeout: u64,
}
```

Note: `k8s_auth` must be included — `console_node_ws` and `console_session_ws` stream directly from Kubernetes pods and need the site's K8s credentials, which can be certificate-based (`K8sAuth::Native`) or Vault-delegated (`K8sAuth::Vault`). These are not just a URL.

Every handler currently calls `state.infra_context()` once. With multi-site, the lookup must happen at request time. An Axum extractor can centralize this:

```rust
// Sketch — extracts the right SiteBackend from ?site= (defaults to default_site)
pub struct SiteContext(pub Arc<SiteBackend>);

impl FromRequestParts<Arc<ServerState>> for SiteContext { ... }
```

Handler signatures gain one parameter; the lookup and 400 error for unknown sites are in one place.

**Estimate**: ~80 lines of extractor code + one added parameter per handler.

---

## Protocol Placement

| Option | GET? | WebSocket? | Body collision? | Breaking change? |
|---|---|---|---|---|
| Query param `?site=` | Yes | Yes | No | No (optional) |
| Header `X-Manta-Site` | Yes | Yes | No | No (optional) |
| JSON body field | No | No | Yes | Yes |
| URL path `/api/v1/{site}/...` | Yes | Yes | No | Yes (all routes change) |

**Query param** is the best fit: works across all HTTP verbs and WebSocket upgrades, can be made optional with the server's `default_site` as fallback (zero breaking change for existing single-site clients), and is visible in logs.

---

## Security Implication

A multi-site server that accepts `?site=eiger` from any authenticated caller gives every token-holder access to every configured site. Because the centralized Keycloak issues tokens valid at all sites, this is expected behaviour — any holder of a valid JWT can already access any site directly. The manta server does not widen the attack surface.

What the server should still do: validate the token against the *target* site's backend, not the default site. Currently `validate_api_token` is called implicitly on the first real API call — there is no explicit pre-flight check in the manta layer. For multi-site this matters because a request could fail midway through a handler on a site whose backend happens to be unreachable, rather than failing fast at token validation. An explicit early `validate_api_token` call against the resolved `SiteBackend` before entering handler logic is the right fix.

---

## Startup Behavior in Multi-site Mode

Currently `manta serve` reads one site at startup and fails fast if the site is unknown. In multi-site mode:
- Does startup fail if any configured site is unreachable? (strict — bad for partial outages)
- Or does it start with whatever sites are reachable? (lenient — harder to detect misconfig)
- What happens when a client requests an unconfigured site name — 400 or 503?

These are operational questions that need answers before the implementation, not after.

---

## Recommendation

The JWT gate passes. Multi-site routing is architecturally viable. The remaining work breaks into two tracks:

**Track 1 — required regardless of multi-site (do first)**

Remove the env-var SOCKS5 coupling in csm-rs. This is a pre-existing correctness issue: `set_var` before the tokio runtime is a workaround, not a design. Centralise `reqwest::Client` construction in csm-rs so `socks5_proxy` is passed explicitly. This unblocks multi-site and also makes the single-site server cleaner.

**Track 2 — the multi-site feature itself**

1. Add `socks5_proxy: Option<String>` and `k8s_auth: Option<K8sAuth>` to `InfraContext` / `SiteBackend` (after Track 1).
2. Refactor `ServerState` from a single backend to `HashMap<String, SiteBackend>`.
3. Add an Axum extractor that reads `?site=` (optional, defaults to `default_site`), looks up the site, and produces the resolved `InfraContext`. Add an explicit `validate_api_token` call against the target site inside the extractor.
4. Thread the extractor into all 47 handler signatures (one-line change each).
5. Update `manta serve` startup to load all configured sites, not just one.

**Deployment model decision**

With the shared-Keycloak confirmation, both models are now valid:

| Model | Trade-off |
|---|---|
| **One server per site** | Simplest. CLI picks the right manta URL per site from config (same pattern as `shasta_base_url`). No `site_name` in the API. No multi-site refactor. Appropriate if each site is managed independently. |
| **One shared multi-site server** | One process, one TLS cert, one URL. Operators pass `?site=alps`. Requires Track 1 + Track 2 above. Appropriate if the team manages all sites centrally and wants a single control plane. |

**Suggested order**: ship Track 1 (SOCKS5 refactor in csm-rs) first as a standalone improvement, then decide which deployment model to target before starting Track 2.
