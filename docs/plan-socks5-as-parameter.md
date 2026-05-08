# Plan: Thread SOCKS5 Proxy as an Explicit Function Parameter

## Rule

Every function in csm-rs and ochami-rs that builds an HTTP client receives
`socks5_proxy: Option<&str>` as an explicit parameter. No environment-variable
reads. No implicit global state.

## Why

- `std::env::set_var` is `unsafe` in a multi-threaded process (Rust 1.66+).
- The process-global `SOCKS5` env var prevents per-site proxy configuration
  in a future multi-site server.
- Explicit parameters make the dependency visible and testable.

---

## Scope

| Repository | Occurrences | Files |
|---|---|---|
| csm-rs | 147 | 34 |
| ochami-rs | 93 | ~10 |
| manta-backend-dispatcher | 0 (constructor change only) | 1 |
| manta | 0 (removal of `set_var`) | 1 |

---

## Step 1 — csm-rs: add `socks5_proxy` to `Csm` struct

`csm-rs/src/backend_connector/mod.rs`:

```rust
#[derive(Clone)]
pub struct Csm {
    pub(crate) base_url: String,
    pub(crate) root_cert: Vec<u8>,
    pub(crate) socks5_proxy: Option<String>,
}

impl Csm {
    pub fn new(base_url: &str, root_cert: &[u8], socks5_proxy: Option<&str>) -> Self {
        Self {
            base_url: base_url.to_string(),
            root_cert: root_cert.to_vec(),
            socks5_proxy: socks5_proxy.map(str::to_owned),
        }
    }
}
```

---

## Step 2 — csm-rs: update every http_client function signature

**Every function that builds a `reqwest::Client`** gains `socks5_proxy: Option<&str>`.
The env-var block is replaced with direct use of the parameter.

Before (example from `bss/http_client.rs`):
```rust
pub async fn get(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xnames: &[String],
) -> Result<Vec<BootParameters>, Error> {
    let client_builder = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);
    let client = if std::env::var("SOCKS5").is_ok() {
        let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
        client_builder.proxy(socks5proxy).build()?
    } else {
        client_builder.build()?
    };
    ...
}
```

After:
```rust
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
    ...
}
```

Apply this to all 34 files. The affected modules are:

- `bss/http_client.rs`
- `pcs/transitions/http_client.rs`
- `pcs/power_status/http_client.rs`
- `pcs/power_cap/http_client.rs`
- `hsm/group/http_client.rs`
- `hsm/component/http_client.rs`
- `hsm/memberships/http_client.rs`
- `hsm/component_status/http_client/mod.rs`
- `hsm/hw_inventory/ethernet_interfaces/http_client.rs`
- `hsm/hw_inventory/hw_component/http_client.rs`
- `hsm/hw_inventory/redfish_endpoint/http_client.rs`
- `hsm/service/values/role/http_client.rs`
- `cfs/session/http_client/v2/mod.rs`
- `cfs/session/http_client/v3/mod.rs`
- `cfs/component/http_client/v2/mod.rs`
- `cfs/component/http_client/v3/mod.rs`
- `cfs/configuration/http_client/v2/mod.rs`
- `cfs/configuration/http_client/v3/mod.rs`
- `cfs/health.rs`
- `bos/template/http_client/v1/mod.rs`
- `bos/template/http_client/v2/mod.rs`
- `bos/session/http_client/v1/mod.rs`
- `bos/session/http_client/v2/mod.rs`
- `ims/image/http_client/mod.rs`
- `ims/job/http_client.rs`
- `ims/recipe/http_client.rs`
- `ims/public_keys.rs`
- `capmc/http_client.rs`
- `common/csm.rs` (`process_get_http_request`)
- `common/authentication.rs`
- `common/gitea.rs`

**Vault (`common/vault.rs`)**: same change but no `add_root_certificate` — the client builder starts without a root cert:
```rust
pub async fn auth_oidc_jwt(
    vault_base_url: &str,
    shasta_token: &str,
    site_name: &str,
    socks5_proxy: Option<&str>,
) -> Result<String, Error> {
    let client_builder = reqwest::Client::builder();
    let client = match socks5_proxy {
        Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
        None => client_builder.build()?,
    };
    ...
}
```

**Kubernetes (`common/kubernetes.rs`)**: `get_client` uses `kube::Client`, not reqwest.
The proxy is set differently:

```rust
pub async fn get_client(
    k8s_api_url: &str,
    shasta_k8s_secrets: Value,
    socks5_proxy: Option<&str>,
) -> Result<kube::Client, Error> {
    ...
    if let Some(proxy_addr) = socks5_proxy {
        config.proxy_url = Some(proxy_addr.parse::<Uri>().map_err(|_| {
            Error::Message("Could not parse socks5_proxy".to_string())
        })?);
    }
    ...
}
```

**IMS S3 client (`ims/s3_client.rs`)**: uses the AWS SDK with `HyperClientBuilder`.
Add `socks5_proxy: Option<&str>` to the function and replace the env-var read:

```rust
// Before
if let Ok(socks5_env) = std::env::var("SOCKS5") { ... }

// After
if let Some(socks5_addr) = socks5_proxy { ... }
```

---

## Step 3 — csm-rs: update all `Csm` trait impl methods

Every `impl SomeTrait for Csm` method that calls an http_client function must
pass `self.socks5_proxy.as_deref()` as the new argument.

Example:
```rust
// Before
async fn get_all_bootparameters(&self, token: &str) -> Result<Vec<BootParameters>, Error> {
    bss::http_client::get_all(token, &self.base_url, &self.root_cert).await
}

// After
async fn get_all_bootparameters(&self, token: &str) -> Result<Vec<BootParameters>, Error> {
    bss::http_client::get_all(token, &self.base_url, &self.root_cert, self.socks5_proxy.as_deref()).await
}
```

All callers of `common::kubernetes::get_client` and `ims::s3_client` functions
inside Csm impl methods pass `self.socks5_proxy.as_deref()` the same way.

---

## Step 4 — ochami-rs: same as Steps 1–3

ochami-rs has no Kubernetes or S3 usage, so only the reqwest pattern applies.

1. Add `socks5_proxy: Option<String>` to `Ochami` struct, update `Ochami::new`.
2. Add `socks5_proxy: Option<&str>` to every http_client function (~93 occurrences across ~10 files).
3. Update every `Ochami` impl method to pass `self.socks5_proxy.as_deref()`.

---

## Step 5 — manta-backend-dispatcher: update `StaticBackendDispatcher::new`

```rust
// Before
pub fn new(backend_type: &str, base_url: &str, root_cert: &[u8]) -> Result<Self, Error>

// After
pub fn new(
    backend_type: &str,
    base_url: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,
) -> Result<Self, Error>
```

Pass `socks5_proxy` through to `Csm::new` and `Ochami::new`.

---

## Step 6 — manta: update `main.rs`, `InfraContext`, and `ServerState`

Pass `socks5_proxy` to the dispatcher:

```rust
let socks5_proxy = site_details_value.socks5_proxy.as_deref();

let backend = StaticBackendDispatcher::new(
    backend_tech.as_str(),
    &shasta_api_url,
    &shasta_root_cert,
    socks5_proxy,
)?;
```

`InfraContext` and `ServerState` also need `socks5_proxy` because some service
functions and CLI commands call csm-rs http_client functions directly (bypassing
the dispatcher's `Csm`/`Ochami` struct). These include:
- `csm_rs::node::utils::get_node_details` (called from `service/cluster.rs` and `service/node.rs`)
- `csm_rs::ims::public_keys::http_client::v3::get_single` (called from `cli/commands/apply_ephemeral_env.rs`)
- `csm_rs::ims::job::http_client::post_customize` (called from `cli/commands/apply_ephemeral_env.rs`)
- `csm_rs::common::gitea::http_client::get_all_refs` (called from `cli/commands/validate_local_repo.rs`)
- `csm_rs::common::gitea::http_client::get_commit_details` (called from `cli/commands/validate_local_repo.rs`)

```rust
// InfraContext gains:
pub socks5_proxy: Option<&'a str>,

// ServerState gains:
pub socks5_proxy: Option<String>,

// ServerState::infra_context() propagates it:
socks5_proxy: self.socks5_proxy.as_deref(),
```

---

## Execution Order

```
Step 1  csm-rs: Csm struct
Step 2  csm-rs: all http_client functions (compiler drives completeness — build fails on any missed call)
Step 3  csm-rs: Csm impl methods — cargo build csm-rs passes
Step 4  ochami-rs: full same sequence — cargo build ochami-rs passes
Step 5  manta-backend-dispatcher — cargo build passes
Step 6  manta — update StaticBackendDispatcher::new, InfraContext, ServerState;
        add socks5_proxy to apply_ephemeral_env::exec and validate_local_repo::exec;
        cargo build manta passes
```

Build each crate to completion before moving to the next. The compiler
will report every missed call site as a type error, making the change
self-verifying.
