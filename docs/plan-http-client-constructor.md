# Plan: HTTP Client Constructor Refactor

## Problem

Every function in csm-rs and ochami-rs that makes an HTTP request rebuilds a
`reqwest::Client` from scratch. The reqwest documentation is explicit: a
`Client` holds a connection pool and should be created once per application and
reused. The current code creates one per *call*, which means:

- No TCP connection reuse — every request opens a new TLS handshake.
- 103 identical 3-5 line blocks duplicated across 21 csm-rs files.
- 87 identical blocks across 15 ochami-rs files (plus obsolete env-var reads).
- Every http_client function carries 3 parameters (`base_url`, `root_cert`,
  `socks5_proxy`) that are constant for the lifetime of a `Csm`/`Ochami`
  instance.

---

## Proposed type: `BackendClient`

A thin wrapper that owns a built `reqwest::Client` and the backend base URL.
Built once at startup, cloned cheaply (reqwest::Client is Arc-backed).

```rust
pub struct BackendClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
}

impl BackendClient {
    pub fn new(
        base_url: &str,
        root_cert: &[u8],
        socks5_proxy: Option<&str>,
    ) -> Result<Self, Error> {
        let mut builder = reqwest::Client::builder().use_rustls_tls();
        if !root_cert.is_empty() {
            builder = builder.add_root_certificate(
                reqwest::Certificate::from_pem(root_cert)?
            );
        }
        let http = match socks5_proxy {
            Some(proxy) => builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
            None => builder.build()?,
        };
        Ok(Self { http, base_url: base_url.to_string() })
    }
}
```

Two notes on the constructor:

1. **`use_rustls_tls()`** is called unconditionally. Both csm-rs and ochami-rs
   already depend on reqwest with `features = ["rustls-tls"]` and no other TLS
   backend, so this is explicit rather than a behaviour change. It also
   eliminates the inconsistency where ochami-rs added `.use_rustls_tls()` and
   csm-rs did not.

2. **Empty root cert**: when `root_cert` is `&[]` (the fallback in `main.rs`
   when the CA file is missing), `from_pem` would fail. Skipping
   `add_root_certificate` in that case falls back to the system trust store,
   which matches the intent of the "proceeding without it" warning.

---

## Where `BackendClient` lives

`manta-backend-dispatcher/src/client.rs`, re-exported from the crate root.

Rationale: `manta-backend-dispatcher` already depends on reqwest (its
`Error::NetError` variant wraps `reqwest::Error`). Placing `BackendClient`
there means one canonical definition shared by csm-rs, ochami-rs, and manta,
with no new dependency edges.

---

## How `Csm` and `Ochami` change

```rust
// Before
#[derive(Clone)]
pub struct Csm {
    pub(crate) base_url: String,
    pub(crate) root_cert: Vec<u8>,
    pub(crate) socks5_proxy: Option<String>,
}
impl Csm {
    pub fn new(base_url: &str, root_cert: &[u8], socks5_proxy: Option<&str>) -> Self

// After
#[derive(Clone)]  // reqwest::Client is Arc-backed, so Clone is cheap
pub struct Csm {
    pub(crate) client: BackendClient,
}
impl Csm {
    pub fn new(
        base_url: &str,
        root_cert: &[u8],
        socks5_proxy: Option<&str>,
    ) -> Result<Self, Error> {
        Ok(Self { client: BackendClient::new(base_url, root_cert, socks5_proxy)? })
    }
}
```

`Ochami` follows the same pattern.

---

## How http_client function signatures change

```rust
// Before (example from bss/http_client.rs)
pub async fn get(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xnames: &[String],
) -> Result<Vec<BootParameters>, Error> {
    let client_builder = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);
    let client = match socks5_proxy {
        Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
        None => client_builder.build()?,
    };
    let url = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);
    ...
}

// After
pub async fn get(
    client: &BackendClient,
    shasta_token: &str,
    xnames: &[String],
) -> Result<Vec<BootParameters>, Error> {
    let url = format!("{}/bss/boot/v1/bootparameters", client.base_url);
    ...
}
```

The 4-line client-build block is deleted. `shasta_base_url` becomes
`client.base_url`. No other logic changes.

---

## How backend_connector trait impls change

```rust
// Before
async fn get_all_bootparameters(&self, token: &str) -> Result<Vec<BootParameters>, Error> {
    bss::http_client::get_all(token, &self.base_url, &self.root_cert, self.socks5_proxy.as_deref()).await
}

// After
async fn get_all_bootparameters(&self, token: &str) -> Result<Vec<BootParameters>, Error> {
    bss::http_client::get_all(&self.client, token).await
}
```

---

## Impact on `StaticBackendDispatcher::new`

```rust
// Before
"csm" => Ok(Self::CSM(Csm::new(base_url, root_cert, socks5_proxy))),

// After — Csm::new now returns Result
"csm" => Ok(Self::CSM(Csm::new(base_url, root_cert, socks5_proxy)?)),
```

`StaticBackendDispatcher::new` already returns `Result<Self, Error>`, so the
`?` propagates cleanly with no other changes.

---

## Special cases excluded from `BackendClient`

| Location | Why excluded | Action |
|----------|--------------|--------|
| `csm-rs: cfs/health.rs` | `connect_timeout(3s)`, no cert, no proxy — a lightweight connectivity probe | Keep as a free `reqwest::Client::builder()` call inside the function; it is intentionally different |
| `csm-rs: common/vault.rs` | No root cert (Vault uses the system TLS store); different base URL per call | Keep as-is |
| `csm-rs: common/kubernetes.rs` | Uses `kube::Client`, not reqwest | No change |
| `csm-rs: ims/s3_client.rs` | AWS SDK (`aws-sdk-s3`) with `HyperClientBuilder`; `s3_auth` uses reqwest but has a distinct URL path | `s3_auth` can be migrated to `BackendClient`; the rest stays as AWS SDK calls |

---

## ochami-rs additional scope

ochami-rs was never migrated from the env-var SOCKS5 pattern. The constructor
refactor absorbs that migration:

- All 15 http_client files gain `client: &BackendClient` as the first parameter.
- All `std::env::var("SOCKS5")` reads are deleted (93 occurrences across 15 files).
- `.use_rustls_tls()` is no longer called per-function — `BackendClient::new`
  handles it.
- `Ochami::new` becomes `Result<Self, Error>`.

---

## Impact on manta

### `StaticBackendDispatcher::new`
Add `?` to `Csm::new` and `Ochami::new` calls (shown above). Done.

### External csm-rs call sites

A handful of functions in manta call csm-rs http_client functions directly,
outside the dispatcher (`service/node.rs`, `service/cluster.rs`,
`cli/commands/apply_ephemeral_env.rs`, `cli/commands/validate_local_repo.rs`).
After the refactor they must supply a `&BackendClient`.

**Approach**: build a `BackendClient` at the call site using the infra context
fields.

```rust
// service/node.rs — after refactor
use manta_backend_dispatcher::client::BackendClient;

let client = BackendClient::new(
    infra.shasta_base_url,
    infra.shasta_root_cert,
    infra.socks5_proxy,
)?;
csm_rs::node::utils::get_node_details(&client, token, node_list).await
```

This creates a fresh `reqwest::Client` (and connection pool) per service call,
not per inner HTTP request. That is a strict improvement on today.

**Follow-up (out of scope here)**: add `csm_client: BackendClient` to
`InfraContext` and `ServerState`, built once at startup, for full connection
reuse. That removes the remaining per-service-call client construction and
eliminates `shasta_base_url + shasta_root_cert + socks5_proxy` as separate
fields in `InfraContext` (they become derivable from the client). Flag for a
future plan.

---

## Scope summary

| Crate | Client builds eliminated | Function signatures simplified | New external dep |
|-------|--------------------------|-------------------------------|-----------------|
| csm-rs | 103 | ~207 | `manta-backend-dispatcher::BackendClient` (already a dep) |
| ochami-rs | 87 + 93 env-var reads | ~103 | same |
| manta-backend-dispatcher | — | — | none (reqwest already used) |
| manta | 0 | minor call-site changes | none |

---

## Execution order

```
Step 1   manta-backend-dispatcher: add src/client.rs with BackendClient
         re-export from lib.rs — no existing code breaks

Step 2   csm-rs: update Csm struct and Csm::new → Result<Self, Error>
         update StaticBackendDispatcher::new to add ? on Csm::new call

Step 3   csm-rs: update all 21 http_client files
         (token, base_url, root_cert, socks5_proxy) → (client, token)
         compiler reports every missed call site as E0061

Step 4   csm-rs: update all 13 backend_connector impl files
         self.base_url / self.root_cert / self.socks5_proxy.as_deref() → &self.client

Step 5   cargo check csm-rs passes (0 errors)

Step 6   ochami-rs: update Ochami struct, Ochami::new → Result<Self, Error>
         update StaticBackendDispatcher::new to add ? on Ochami::new call

Step 7   ochami-rs: update all 15 http_client files
         remove env-var reads, adopt (client, token) pattern

Step 8   ochami-rs: update all backend_connector impl files

Step 9   cargo check ochami-rs passes (0 errors)

Step 10  manta: update external call sites (construct BackendClient locally)
         cargo test manta passes (317+ tests)
```

The compiler drives completeness: any missed call site is a type error.
No step can be silently skipped.
